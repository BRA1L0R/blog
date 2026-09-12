---
title: "Building an AM transmitter with an Arduino and a jumper"
description: "Not all RF is done through meticolously matched impedances and fancy antennas."
date: "2026-08-15"
---

<video width="100%" height="auto" controls>
  <source src="/static/media/arduino-radio.mp4" type="video/mp4">
Your browser does not support the video tag.
</video>

# How this works

The carrier is a 1 MHz square wave out of Timer 2, running in CTC mode and toggling D3 on every compare match. A jumper wire on that pin is the antenna.

Modulation is where it gets fun. There's no analog modulator, Timer 1 just gates the carrier on and off at 31.25 kHz, the audio sample rate. Every 32 µs the overflow interrupt switches the antenna pins on and sets a compare value from the current audio sample, the compare interrupt switches them back off. The longer the carrier stays on, the louder the sample. A sample of 128 keeps it on half the time, which is silence. The radio's envelope detector averages all that switching out and hands you back your audio.

The audio itself arrives over the UART at 500 kbaud, raw 8-bit samples at 31250 per second. The RX interrupt drops each byte into a 256-byte ring buffer, the timer interrupt drains it. If the buffer fills up faster than it drains, `uartOverruns` ticks up. If it runs dry, the last sample just repeats and `bufferUnderruns` goes up instead.

The host-side implementation of the pipewire loopback for feeding the audio into the UART is left as an exercise for the reader.

# The arduino code

<details>

```c
// OCR2 values for available Transmit frequencies: OCR2A/B = 8000/Ft[Khz] - 1
#define F_8MHZ 0
#define F_4MHZ 1
#define F_2666 2
#define F_2000 3
#define F_1600 4
#define F_1333 5
#define F_1143 6
#define F_1000 7
#define F_880 8
#define F_800 9
#define F_727 10
#define F_666 11
#define F_615 12
#define F_571 13
#define F_533 14
#define F_500 15
#include <Arduino.h>
#include <avr/interrupt.h>

#define ANTENNA_DDR DDRD
#define ANTENNA_BIT PD3 // Arduino D3, OC2B

#define UART_BAUD 500000UL
#define AUDIO_RATE 31250UL

#if (UART_BAUD < (AUDIO_RATE * 10UL))
#error "UART_BAUD is too slow for AUDIO_RATE with 8-N-1 framing"
#endif

#if ((F_CPU % AUDIO_RATE) != 0)
#error "AUDIO_RATE must divide F_CPU exactly"
#endif

// Timer 2 carrier:
//
// f_carrier = F_CPU / (2 * (OCR2A + 1))
//
// 0  = 8 MHz
// 1  = 4 MHz
// 3  = 2 MHz
// 4  = 1.6 MHz
// 7  = 1 MHz
// 9  = 800 kHz
// 15 = 500 kHz

#define CARRIER_OCR2A 7

// Timer 1 runs at AUDIO_RATE.
//
// With a 16 MHz clock and no prescaler:
//
// TOP = F_CPU / AUDIO_RATE - 1
//
// At 31.25 kHz, TOP = 511.

#define MODULATION_TOP ((F_CPU / AUDIO_RATE) - 1UL)

volatile uint8_t audioBuffer[256];
volatile uint8_t audioReadPosition = 0;
volatile uint8_t audioWritePosition = 0;

volatile uint8_t lastSample = 128;

volatile uint16_t uartOverruns = 0;
volatile uint16_t bufferUnderruns = 0;

static inline uint16_t sampleToCompare(uint8_t sample) {
  // MODULATION_TOP is 511 at 31.25 kHz. For samples 0..254,
  // floor(sample * 511 / 255) is exactly sample * 2. Sample 255 maps to
  // 511. This avoids a costly 32-bit division in the timer interrupt while
  // preserving the complete, unclamped 0..511 modulation range.
  static_assert(MODULATION_TOP == 511UL,
                "sampleToCompare requires a Timer 1 TOP of 511");
  return ((uint16_t)sample << 1) + (sample == 255);
}

static void setupCarrierTimer() {
  // Initially keep the RF pin disconnected/high impedance.
  ANTENNA_DDR &= ~_BV(ANTENNA_BIT);
  PORTD &= ~_BV(PD3);

  // Stop and reset Timer 2.
  TCCR2A = 0;
  TCCR2B = 0;
  TCNT2 = 0;

  OCR2A = CARRIER_OCR2A;
  OCR2B = CARRIER_OCR2A;

  // CTC mode:
  // Timer counts from 0 through OCR2A.
  //
  // Toggle OC2B on its compare match.
  TCCR2A = _BV(COM2B0) | _BV(WGM21);

  // No clock prescaling.
  TCCR2B = _BV(CS20);
}

static void setupModulationTimer() {
  // Stop and reset Timer 1.
  TCCR1A = 0;
  TCCR1B = 0;
  TCNT1 = 0;

  ICR1 = (uint16_t)MODULATION_TOP;
  OCR1A = sampleToCompare(128);

  // Fast PWM, mode 14:
  //
  // WGM13:0 = 1110
  // TOP = ICR1
  //
  // Timer 1 is only being used for timing. Its hardware PWM output
  // pins are not enabled.
  TCCR1A = _BV(WGM11);

  TCCR1B = _BV(WGM13) | _BV(WGM12) | _BV(CS10);

  // Interrupt at the beginning of each modulation period and when
  // Timer 1 reaches OCR1A.
  TIMSK1 = _BV(TOIE1) | _BV(OCIE1A);
}

static void setupUart() {
  // Disable the Arduino HardwareSerial configuration and configure
  // USART0 directly.
  UCSR0A = 0;
  UCSR0B = 0;
  UCSR0C = 0;

  const uint16_t ubrr = (uint16_t)((F_CPU / (16UL * UART_BAUD)) - 1UL);

  UBRR0H = (uint8_t)(ubrr >> 8);
  UBRR0L = (uint8_t)ubrr;

  // 8 data bits, no parity, 1 stop bit.
  UCSR0C = _BV(UCSZ01) | _BV(UCSZ00);

  // Enable UART receiver and receive-complete interrupt.
  UCSR0B = _BV(RXEN0) | _BV(RXCIE0);
}

void setup() {
  cli();

  setupCarrierTimer();
  setupModulationTimer();
  setupUart();

  sei();
}

ISR(USART_RX_vect) {
  uint8_t status = UCSR0A;
  uint8_t sample = UDR0;

  // Discard bytes with framing, parity or hardware-overrun errors.
  if (status & (_BV(FE0) | _BV(UPE0) | _BV(DOR0))) {
    uartOverruns++;
    return;
  }

  uint8_t nextPosition = audioWritePosition + 1;

  // The buffer is full when advancing the write position would collide
  // with the read position.
  if (nextPosition == audioReadPosition) {
    uartOverruns++;
    return;
  }

  audioBuffer[audioWritePosition] = sample;
  audioWritePosition = nextPosition;
}

ISR(TIMER1_OVF_vect) {
  if (OCR1A == 0) {
    ANTENNA_DDR &= ~_BV(ANTENNA_BIT);
  } else {
    ANTENNA_DDR |= _BV(ANTENNA_BIT);
  }

  if (audioReadPosition != audioWritePosition) {
    lastSample = audioBuffer[audioReadPosition];
    audioReadPosition++;
  } else {
    bufferUnderruns++;
  }

  OCR1A = sampleToCompare(lastSample);
}

ISR(TIMER1_COMPA_vect) {
  ANTENNA_DDR &= ~_BV(ANTENNA_BIT);
}

void loop() {}
```

</details>

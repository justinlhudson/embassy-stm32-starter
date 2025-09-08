/* Linker script for the STM32F413ZH MCU (Nucleo-144 Development Board) */
/* 
 * STM32F413ZH Specifications:
 * - ARM Cortex-M4F core @ up to 100 MHz
 * - 1536 KB (1.5 MB) Flash memory
 * - 320 KB SRAM total (256 KB main SRAM + 64 KB auxiliary SRAM)
 * - LQFP144 package with 114 GPIO pins
 * - USB OTG FS/HS, CAN, multiple UART/SPI/I2C interfaces
 * 
 * Memory mapping:
 * - Flash: 0x08000000 - 0x0817FFFF (1536 KB)
 * - SRAM1: 0x20000000 - 0x2003FFFF (256 KB main SRAM)
 * - SRAM2: 0x10000000 - 0x1000FFFF (64 KB auxiliary SRAM, not used by default)
 */

MEMORY
{
  FLASH (rx)      : ORIGIN = 0x08000000, LENGTH = 1536K
  RAM (rwx)       : ORIGIN = 0x20000000, LENGTH = 256K
}

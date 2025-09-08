/* Linker script for the STM32F446RE MCU (Nucleo-64 Development Board) */
/* 
 * STM32F446RE Specifications:
 * - ARM Cortex-M4F core @ up to 180 MHz
 * - 512 KB Flash memory
 * - 128 KB SRAM
 * - LQFP64 package with 50 GPIO pins
 * - USB OTG FS, CAN, multiple UART/SPI/I2C interfaces
 * 
 * Memory mapping:
 * - Flash: 0x08000000 - 0x0807FFFF (512 KB)
 * - SRAM:  0x20000000 - 0x2001FFFF (128 KB)
 */

MEMORY
{
  FLASH (rx)      : ORIGIN = 0x08000000, LENGTH = 512K
  RAM (rwx)       : ORIGIN = 0x20000000, LENGTH = 128K
}

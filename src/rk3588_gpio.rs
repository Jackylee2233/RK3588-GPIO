// rk3588_gpio.rs (最終驗證版)
// 該驅動程式用於控制 RK3588 SoC 的單一 GPIO 引腳。

use core::ptr::write_volatile;

// --- 已確認的硬體位址與偏移 ---

/// RK3588 GPIO Bank 1 的記憶體基底 L位址。
/// 來源: RK3588 TRM Part1, Chapter 20 GPIO & Linux DTS file.
const GPIO1_BASE: usize = 0xfec20000; // 在DTS aliases中gpio1對應的地址是/pinctrl/gpio@fec20000，故使用此地址

/// GPIO 資料暫存器 (高 16 位)。控制 C0-D7 引腳的輸出電平。
/// 來源: RK3588 TRM Part1, Page 1470, "Registers Summary".
const GPIO_SWPORT_DR_H_OFFSET: usize = 0x0004;

/// GPIO 方向暫存器 (高 16 位)。控制 C0-D7 引腳的輸入/輸出模式。
/// 來源: RK3588 TRM Part1, Page 1470, "Registers Summary".
const GPIO_SWPORT_DDR_H_OFFSET: usize = 0x000C;

/// BUS_IOC (I/O Controller) 的記憶體基底 L位址。
/// 來源: rk3588-orangepi-5-plus.dts, syscon@fd5f0000 node.
const BUS_IOC_BASE: usize = 0xFD5F0000;

/// GPIO1C 組引腳的 IOMUX 功能選擇暫存器 (高位)。
/// 來源: RK3588 TRM (BUS_IOC), Page 984, "Registers Summary".
const BUS_IOC_GPIO1C_IOMUX_SEL_H_OFFSET: usize = 0x0034;


/// 代表一個 GPIO 引腳的驅動程式結構。
pub struct GpioPin {
    pin_num_global: u8, // 全局引腳編號 (0-31), 例如 C4 是 20
    gpio_base: usize,
    bus_ioc_base: usize,
}

impl GpioPin {
    /// 建立一個代表 Orange Pi 5 Plus 板載 LED (GPIO1_C4) 的新實例。
    pub fn new_led() -> Self {
        Self {
            pin_num_global: 20, // GPIO1_C4: C 組是第 3 組 (A=0, B=1, C=2), C4 是該組第 4 個引腳。
                               // 全局索引 = 8(A組) + 8(B組) + 4 = 20
            gpio_base: GPIO1_BASE,
            bus_ioc_base: BUS_IOC_BASE,
        }
    }

    // TODO: 這個函數僅用於測試目的。稍後可移除。
    /// 為測試環境建立一個使用偽造基地址的 GpioPin 實例。
    pub fn new_led_for_test(gpio_base: usize, bus_ioc_base: usize) -> Self {
        Self {
            pin_num_global: 20,
            gpio_base,
            bus_ioc_base,
        }
    }

    /// 將引腳的硬體功能設定為 GPIO 模式。
    /// 這是操作 GPIO 的第一步，必須先執行。
    /// 資訊來源: RK3588 TRM (BUS_IOC), Page 984, 990.
    pub fn set_function_gpio(&self) {
        // 計算 IOMUX 控制暫存器的完整記憶體位址
        let iomux_reg_addr = self.bus_ioc_base + BUS_IOC_GPIO1C_IOMUX_SEL_H_OFFSET;
        
        // GPIO1_C4 對應 gpio1c4_sel, 位於該暫存器的 bits [3:0]。
        // 每個引腳的 IOMUX 設定佔用 4 個位元。
        // C4 在 (C4, C5, C6, C7) 這半組中的索引是 0。
        let pin_bit_offset = (self.pin_num_global % 4) * 4;

        // 使用 GRF/IOC 的寫入遮罩機制: 高 16 位為遮罩，低 16 位為數值。
        // 1. 準備寫入遮罩：我們要修改 bits [3:0]，所以遮罩是 0b1111。
        let write_mask = 0b1111 << (16 + pin_bit_offset);
        // 2. 準備數值：GPIO 功能對應的值是 0。
        let value = 0b0000 << pin_bit_offset;
        
        unsafe {
            // 寫入暫存器以改變引腳功能
            write_volatile(iomux_reg_addr as *mut u32, write_mask | value);
        }
    }

    /// 將引腳的方向設定為輸出 (Output) 模式。
    /// 資訊來源: RK3588 TRM Part1, Page 1470, 1471.
    pub fn set_as_output(&self) {
        // 引腳 20 屬於高 16 位 (16-31)，因此使用 _DDR_H 暫存器。
        let ddr_reg_addr = self.gpio_base + GPIO_SWPORT_DDR_H_OFFSET;
        // 在高 16 位組內，引腳 20 的本地索引是 4 (20 - 16 = 4)。
        let local_pin_num = self.pin_num_global - 16;

        // 使用 GPIO 的寫入遮罩機制。
        // 1. 準備遮罩，致能對 local_pin_num 的寫入。
        let mask = 1 << (16 + local_pin_num);
        // 2. 準備數值，將 local_pin_num 對應位設為 1 (Output)。
        let value = 1 << local_pin_num;
        
        unsafe {
            write_volatile(ddr_reg_addr as *mut u32, mask | value);
        }
    }

    /// 設置引腳為高電平 (點亮 LED)。
    pub fn set_high(&self) {
        let dr_reg_addr = self.gpio_base + GPIO_SWPORT_DR_H_OFFSET;
        let local_pin_num = self.pin_num_global - 16;
        
        let mask = 1 << (16 + local_pin_num);
        let value = 1 << local_pin_num; // 1 -> High
        
        unsafe {
            write_volatile(dr_reg_addr as *mut u32, mask | value);
        }
    }

    /// 設置引腳為低電平 (熄滅 LED)。
    pub fn set_low(&self) {
        let dr_reg_addr = self.gpio_base + GPIO_SWPORT_DR_H_OFFSET;
        let local_pin_num = self.pin_num_global - 16;

        let mask = 1 << (16 + local_pin_num);
        let value = 0 << local_pin_num; // 0 -> Low
        
        unsafe {
            write_volatile(dr_reg_addr as *mut u32, mask | value);
        }
    }
}
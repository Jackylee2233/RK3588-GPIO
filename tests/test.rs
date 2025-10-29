#![no_std]
#![no_main]
#![feature(used_with_arg)]

extern crate alloc;
extern crate bare_test;

// Note: The panic handler and global allocator are provided by the `bare_test`
// framework's dependencies (sparreal_kernel), so they are not needed here.

#[bare_test::tests]
mod tests {
    use bare_test::println;
    // Use the correct crate name as defined in Cargo.toml
    use gpio_rk3588_fresh_led::rk3588_gpio::GpioPin;
    use log::info;

    #[test]
    fn it_works() {
        info!("This is a test log message.");
        let a = 2;
        let b = 2;
        assert_eq!(a + b, 4);
        println!("it_works test passed!");
    }

    /// GPIO 驅動程式的冒煙測試。
    ///
    /// 這個測試會呼叫所有公開的 GPIO 函數，以確保它們能夠被編譯且可執行。
    /// 它不會驗證實際的硬體狀態（例如引腳電平），因為這需要在目標硬體上
    /// 透過外部測量工具（如示波器）來完成。
    ///
    /// 如果在裸機環境中執行此測試且沒有發生崩潰，即可視為測試通過。
    #[test]
    fn gpio_smoke_test() {
        println!("Running GPIO smoke test...");

        // 1. 為 LED 建立一個 GpioPin 實例。
        let led_pin = GpioPin::new_led();
        println!("- GpioPin created.");

        // 2. 將引腳功能設定為 GPIO。
        led_pin.set_function_gpio();
        println!("- Function set to GPIO.");

        // 3. 將引腳方向設定為輸出。
        led_pin.set_as_output();
        println!("- Direction set to output.");

        // 4. 將引腳設定為高電平。
        led_pin.set_high();
        println!("- Pin set to high.");

        // 5. 將引腳設定為低電平。
        led_pin.set_low();
        println!("- Pin set to low.");

        println!("GPIO smoke test completed successfully.");
        // 如果程式能執行到這裡沒有崩潰，我們就認為測試通過。
        assert!(true);
    }
}

// const LOWER_BRIGHTNESS_BOUND: u16 = 5000;
// const UPPER_BRIGHTNESS_BOUND: u16 = 12000;
    // let lights_clone = lights.clone();
    // loop {
    //     if (*is_terminating).load(std::sync::atomic::Ordering::Acquire) {
    //         break;
    //     }

    //     tx.send(Request {
    //         options: req_options.clone(),
    //         message: Message::GetColor,
    //         target: broadcast_address.to_string(),
    //     })
    //     .unwrap();

    //     req_options.increment_sequence();

    //     sleep(Duration::from_millis(800)).await;

    //     let lights = lights.read().await;
    //     for (addr, light) in lights.iter() {
    //         // set light brightness to + or - 10% of its prev value
    //         let light = light.read().await;

    //         let mut brightness = match light.brightness {
    //             Some(brightness) => brightness,
    //             None => continue,
    //         };

    //         if brightness < LOWER_BRIGHTNESS_BOUND {
    //             brightness = LOWER_BRIGHTNESS_BOUND;
    //         } else if brightness > UPPER_BRIGHTNESS_BOUND {
    //             brightness = UPPER_BRIGHTNESS_BOUND;
    //         }

    //         let mut amount_to_add = (brightness as f32 * 0.08) as i32;

    //         if random() {
    //             amount_to_add *= -1;
    //         }

    //         let new_brightness = brightness as i32 + amount_to_add;

    //         // tx.send(Request {
    //         //     options: req_options.clone(),
    //         //     message: Message::SetColor {
    //         //         reserved_6: 1,
    //         //         hue: light.hue.unwrap(),
    //         //         saturation: light.saturation.unwrap(),
    //         //         brightness: new_brightness as u16,
    //         //         kelvin: light.kelvin.unwrap(),
    //         //         duration_ms: 450,
    //         //     },
    //         //     target: Some(addr.to_string()),
    //         // })
    //         // .unwrap();

    //         req_options.increment_sequence();
    //     }

    //     sleep(Duration::from_millis(350)).await;
    // }
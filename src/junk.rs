// frame width should be scaled based on the smaller dimension
        let base = original_content_size.min_dim();
        let frame_width: Sides = options.frame_width * base;
        let margin: Sides = Sides::uniform((margin_factor * base as f32) as u32);

        // let frame_width = options.frame_width * content_size;
        // crate::debug!(&frame_width);
        // crate::debug!(&margin);
        // crate::debug!(frame_width + margin);
        // crate::debug!(content_size);

        // let content_size = content_size + frame_width + margin;
        // crate::debug!(content_size);


        // let default_output_size = original_content_size + frame_width + margin;
        let default_output_size = original_content_size.scale_by(1.0 + options.frame_width);
        let output_size = {
            // let content_size = content_size + frame_width;
            // let margin = (1.0 - scale_factor) * base as f32;
            // + margin;
            // let mut default_output_size = content_size * (1.0 / scale_factor);

            // let mut output_size_bounds: OutputSize = default_output_size.into();
            let output_size_bounds =
                OutputSize::from(default_output_size).min(options.output_size_bounds);

            // scale the default content size to bounds
            // let before = default_output_size.clone();
            let output_size =
                default_output_size.scale_to(output_size_bounds, types::ResizeMode::Contain);
            // assert_eq!(before, default_output_size);

            match options.output_size.min(options.output_size_bounds) {
                // if no absolute dimensions, choose default
                OutputSize {
                    width: None,
                    height: None,
                } => output_size,
                // if underspecified, scale by default content aspect ratio
                OutputSize {
                    width: None,
                    height: Some(height),
                } => {
                    let ratio = height as f64 / output_size.height as f64;
                    let width = output_size.width as f64 * ratio;
                    Size {
                        width: width as u32,
                        height,
                    }
                }
                OutputSize {
                    width: Some(width),
                    height: None,
                } => {
                    let ratio = width as f64 / output_size.width as f64;
                    let height = output_size.height as f64 * ratio;
                    Size {
                        width,
                        height: height as u32,
                    }
                }
                // if only absolute values, nothing to be done
                OutputSize {
                    width: Some(width),
                    height: Some(height),
                } => Size { width, height },
            }
        };


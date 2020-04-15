use crate::wired_logic::*;

pub trait RenderFrames {
    fn render_frames(
        &mut self,
        image: &image::RgbaImage,
        delay: std::time::Duration,
    ) -> Vec<image::Frame>;
}

impl RenderFrames for Circuit {
    fn render_frames(
        &mut self,
        img: &image::RgbaImage,
        delay: std::time::Duration,
    ) -> Vec<image::Frame> {
        let saved_state = self.state.clone();

        let mut states = Vec::new();
        self.state.iter_mut().for_each(|i| *i = 0);

        let mut start;
        while {
            start = states.iter().position(|x| x == &self.state);
            start.is_none()
        } {
            states.push(self.state.clone());
            self.step();
        }

        let mut frames = Vec::new();
        for i in start.unwrap()..states.len() {
            let mut frame = img.clone();
            self.state = states[i].clone();
            self.render(&mut frame);

            let frame = image::Frame::from_parts(
                frame,
                0,
                0,
                image::Delay::from_saturating_duration(delay),
            );
            frames.push(frame);
        }

        self.state = saved_state;

        frames
    }
}

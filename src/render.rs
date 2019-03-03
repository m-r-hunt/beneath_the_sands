use super::physics::Movement;
use quicksilver::graphics::Color;
use quicksilver::lifecycle::Window;
use specs::prelude::*;

#[derive(Default)]
pub struct RenderComponent {
    pub radius: f32,
    pub colour: Color,
}

impl Component for RenderComponent {
    type Storage = VecStorage<Self>;
}

pub struct Render<'a> {
    pub window: &'a mut Window,
}

impl<'a: 'b, 'b> System<'b> for Render<'a> {
    type SystemData = (ReadStorage<'b, Movement>, ReadStorage<'b, RenderComponent>);

    fn run(&mut self, (movement, render): Self::SystemData) {
        self.window
            .clear(quicksilver::graphics::Color::BLACK)
            .unwrap();
        for (movement, render) in (&movement, &render).join() {
            let rect = quicksilver::geom::Circle::new(movement.position, render.radius);
            self.window
                .draw(&rect, quicksilver::graphics::Background::Col(render.colour));
        }
    }
}

use std::time::Duration;

use futures::StreamExt;

use web_sys::HtmlCanvasElement;

use yew::{prelude::*, html, NodeRef, platform::time::interval};

use plotters::prelude::*;
use plotters_canvas::CanvasBackend;

const SIGMA: f64 = 10.;
const RHO: f64 = 28.;
const BETA: f64 = 8. / 3.;
const DELTA: f64 = 0.001;

#[derive(Default)]
struct App {
    canvas_ref: NodeRef,
    data: Vec<(f64, f64, f64)>
}

impl<'a> App {
    fn plot(&'a mut self, backend: CanvasBackend) -> Result<(), Box<dyn std::error::Error>> {
        let root = backend.into_drawing_area();
        root.fill(&RGBColor(32, 32, 32))?;

        let min_x = self.data.iter().min_by(|(x1, _, _), (x2, _, _)| (x1).total_cmp(x2)).unwrap().0;
        let max_x = self.data.iter().max_by(|(x1, _, _), (x2, _, _)| (x1).total_cmp(x2)).unwrap().0;
        let min_y = self.data.iter().min_by(|(_, y1, _), (_, y2, _)| (y1).total_cmp(y2)).unwrap().1;
        let max_y = self.data.iter().max_by(|(_, y1, _), (_, y2, _)| (y1).total_cmp(y2)).unwrap().1;
        let min_z = self.data.iter().min_by(|(_, _, z1), (_, _, z2)| (z1).total_cmp(z2)).unwrap().2;
        let max_z = self.data.iter().max_by(|(_, _, z1), (_, _, z2)| (z1).total_cmp(z2)).unwrap().2;
        let x_axis = (min_x..max_x).step(0.1);
        let y_axis = (min_y..max_y).step(0.1);
        let z_axis = (min_z..max_z).step(0.1);

        let mut chart = ChartBuilder::on(&root)
            .build_cartesian_3d(x_axis, y_axis, z_axis)?;

        chart.with_projection(|mut pb| {
            pb.yaw = 0.1;
            pb.pitch = 0.1;
            pb.scale = 0.9;
            pb.into_matrix()
        });

        chart
            .configure_axes()
            .light_grid_style(WHITE.mix(0.15))
            .max_light_lines(3)
            .draw()?;

        chart
            .draw_series(LineSeries::new(self.data.clone(), &WHITE))?
            .label("Line");

        chart
            .configure_series_labels()
            .border_style(&WHITE)
            .draw()?;

        // To avoid the IO failure being ignored silently, we manually call the present function
        root.present()?;
        Ok(())
    }
}

enum Msg {
    Tick(())
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let time_stream = interval(Duration::from_millis(100)).map(|_| ());
        ctx.link().send_stream(time_stream.map(Msg::Tick));
        Self {
            data: vec![(0., 1., 1.05)],
            ..Default::default()
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Tick(_item) => {
                let mut prev = self.data[self.data.len() - 1];
                for _ in 0..200 {
                    let x = prev.0 + DELTA * (SIGMA * (prev.1 - prev.0));
                    let y = prev.1 + DELTA * (RHO * prev.0 - prev.1 - prev.0 * prev.2);
                    let z = prev.2 + DELTA * (prev.0 * prev.1 - BETA * prev.2);
                    let local_next = (x, y, z);
                    self.data.push(local_next);
                    prev = local_next;
                }

                let canvas = self.canvas_ref.cast::<HtmlCanvasElement>().unwrap();
                let backend = CanvasBackend::with_canvas_object(canvas).expect("cannot find canvas");
                self.plot(backend).unwrap();

                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <html class="dark">
            <div class="flex justify-center">
            <div class="bg-white dark:bg-gray-800 p-4 mx-auto">
                <h1 class="text-3cl font-bold text-center">{"Plot"}</h1>
                <canvas ref={ self.canvas_ref.clone() } width="1200" height="1200">{ "Canvas Error" }</canvas>
            </div>
            </div>
            </html>
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

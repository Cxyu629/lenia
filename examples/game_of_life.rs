use lenia::*;

const SIZE: (u32, u32) = (1280, 720);

fn main() {
    let gol_kernel_core = Mapping::new(Arc::new(|x| {
        if x < 0.25 {
            0.5
        } else if x <= 0.75 {
            1.0
        } else {
            0.0
        }
    }));

    let kernel_shell = KernelShell::new(vec![1.0], gol_kernel_core);
    let kernel_image = KernelImage::new(&kernel_shell, 20, 1.0);

    kernel_image
        .save_image("lenia/assets/kernels/gol_kernel_x10.png")
        .unwrap();

    let lenia_board = LeniaBoard::new(
        LeniaRule::new(
            kernel_shell,
            Mapping::from_type(MappingType::StepGrowth {
                mu: 0.35,
                sigma: 0.07,
            }),
        ),
        SIZE,
        2,
        1.0,
        100,
    );

    lenia_board
        .get_growth_vector()
        .iter()
        .enumerate()
        .for_each(|(i, x)| println!("{i}:{x}"));
    println!("{:#?}", lenia_board.generate_params());

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // uncomment for unthrottled FPS
                // present_mode: bevy::window::PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugin(LeniaRenderPlugin::new(lenia_board))
        .add_plugin(LeniaComputePlugin)
        .run();
}

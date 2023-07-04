use lenia::*;

const SIZE: (u32, u32) = (1280, 720);

fn main() {
    let lenia_kernel_core = Mapping::from_type(MappingType::GaussianCore { alpha: 4.0 });

    let kernel_shell = KernelShell::new(vec![0.5, 2.0 / 3.0, 1.0], lenia_kernel_core);
    let kernel_image = KernelImage::new(&kernel_shell, 36, 1.0);

    kernel_image
        .save_image("lenia/assets/kernels/lenia_kernel_x2.png")
        .unwrap();

    let lenia_board = LeniaBoard::new(
        LeniaRule::new(
            kernel_shell,
            Mapping::from_type(MappingType::GaussianGrowth {
                mu: 0.14,
                sigma: 0.015,
            }),
        ),
        SIZE,
        18,
        0.1,
        100,
    );

    // lenia_board.get_growth_vector().iter().enumerate().for_each(|(i,x)| println!("{i:<2} : {x:.4}"));
    // println!("{:#?}", lenia_board.generate_params());

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

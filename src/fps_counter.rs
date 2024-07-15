use bevy::{
	diagnostic::{DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
	prelude::*,
};

pub struct FpsCounterPlugin;

impl Plugin for FpsCounterPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin))
			.add_systems(Startup, setup)
			.add_systems(PostUpdate, change_text_system);
	}
}

fn setup(mut commands: Commands) {
	commands.spawn((
		TextBundle::from_sections([TextSection::new("x FPS", TextStyle::default())]).with_style(
			Style {
				position_type: PositionType::Absolute,
				bottom: Val::Px(5.0),
				right: Val::Px(15.0),
				..default()
			},
		),
		TextChanges,
	));
}

#[derive(Component)]
struct TextChanges;

fn change_text_system(
	time: Res<Time>,
	diagnostics: Res<DiagnosticsStore>,
	mut query: Query<&mut Text, With<TextChanges>>,
) {
	for mut text in &mut query {
		let mut fps = 0.0;
		if let Some(fps_diagnostic) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
			if let Some(fps_smoothed) = fps_diagnostic.smoothed() {
				fps = fps_smoothed;
			}
		}

		let mut frame_time = time.delta_seconds_f64();
		if let Some(frame_time_diagnostic) =
			diagnostics.get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
		{
			if let Some(frame_time_smoothed) = frame_time_diagnostic.smoothed() {
				frame_time = frame_time_smoothed;
			}
		}

		let num_entities = {
			if let Some(num_entities_diagnostics) =
				diagnostics.get(&EntityCountDiagnosticsPlugin::ENTITY_COUNT)
			{
				if let Some(smoothies) = num_entities_diagnostics.smoothed() {
					smoothies
				} else {
					0.
				}
			} else {
				0.
			}
		};
		let ms_per_entity = frame_time / num_entities;

		text.sections[0].value = [
			format!("{fps:.1} FPS ({frame_time:.1}ms)"),
			format!("Entities: {num_entities:.0}"),
			format!("time / entity: {ms_per_entity:.3}ms"),
		]
		.join("\n");
	}
}

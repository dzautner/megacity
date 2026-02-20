use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::policies::{Policies, Policy};

use super::PoliciesVisible;

pub fn policies_ui(
    mut contexts: EguiContexts,
    mut policies: ResMut<Policies>,
    visible: Res<PoliciesVisible>,
) {
    if !visible.0 {
        return;
    }

    egui::Window::new("Policies")
        .default_open(true)
        .default_width(300.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.small("Press [P] to toggle");
            ui.label(format!(
                "Monthly cost: ${:.0}",
                policies.total_monthly_cost()
            ));
            ui.separator();

            for &policy in Policy::all() {
                let mut active = policies.is_active(policy);
                let cost_str = if policy.monthly_cost() > 0.0 {
                    format!(" (${:.0}/mo)", policy.monthly_cost())
                } else {
                    String::new()
                };
                if ui
                    .checkbox(&mut active, format!("{}{}", policy.name(), cost_str))
                    .changed()
                {
                    policies.toggle(policy);
                }
                ui.label(format!("  {}", policy.description()));
                ui.add_space(4.0);
            }
        });
}

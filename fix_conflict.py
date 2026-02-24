import re

with open('/tmp/worktree-768/crates/simulation/src/plugin_registration.rs', 'r') as f:
    content = f.read()

# Replace conflict markers with both additions
content = content.replace(
    '<<<<<<< HEAD\n    // Bevy diagnostics and trace spans (TEST-031)\n    app.add_plugins(diagnostics::DiagnosticsPlugin);\n=======\n    // Cultural buildings prestige system (SVC-014)\n    app.add_plugins(cultural_buildings::CulturalBuildingsPlugin);\n>>>>>>> ac1fcece (Add cultural buildings prestige system (SVC-014))',
    '    // Bevy diagnostics and trace spans (TEST-031)\n    app.add_plugins(diagnostics::DiagnosticsPlugin);\n\n    // Cultural buildings prestige system (SVC-014)\n    app.add_plugins(cultural_buildings::CulturalBuildingsPlugin);'
)

with open('/tmp/worktree-768/crates/simulation/src/plugin_registration.rs', 'w') as f:
    f.write(content)

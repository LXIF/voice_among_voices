use crate::{SimulationParameters, VoiceNodeIngress};

pub fn node_within_circle(node: &VoiceNodeIngress, sim_params: &SimulationParameters) -> bool {
    let VoiceNodeIngress { x, y, .. } = node;
    let SimulationParameters {
        logical_width,
        logical_height,
        ..
    } = sim_params;

    let node_radius = 2.; // TODO: get this from acual sample length
    let distance_from_center =
        ((x - logical_width / 2.).powi(2) + (y - logical_height / 2.).powi(2)).sqrt();
    let max_distance = logical_width / 2. - node_radius;

    if distance_from_center > max_distance {
        false
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SIM_PARAMS: SimulationParameters = SimulationParameters {
        velocity_cutoff: 0.2,
        force_cutoff: 100.,
        max_distance: 20.,
        force_strength: 3000.,
        linear_damping: 10.,
        logical_height: 100.,
        logical_width: 100.,
        n_collider_vertices: 360,
        friction: 0.5,
        density: 2.,
    };

    #[test]
    fn allows_within_circle() {
        let test_node: VoiceNodeIngress = VoiceNodeIngress {
            x: 98.,
            y: 50.,
            sample: String::new(), // will need to be updated
        };

        assert!(node_within_circle(&test_node, &TEST_SIM_PARAMS));
    }

    #[test]
    fn rejects_on_circle() {
        let test_node: VoiceNodeIngress = VoiceNodeIngress {
            x: 99.,
            y: 50.,
            sample: String::new(), // will need to be updated
        };

        assert!(!node_within_circle(&test_node, &TEST_SIM_PARAMS));
    }

    #[test]
    fn rejects_outside_circle() {
        let test_node: VoiceNodeIngress = VoiceNodeIngress {
            x: 0.,
            y: 0.,
            sample: String::new(), // will need to be updated
        };

        assert!(!node_within_circle(&test_node, &TEST_SIM_PARAMS));
    }
}

use openxr as xr;
use xr::Space;
use crate::openxr_handler::OpenXRHandler;

pub struct OpenXRInput {
    pub action_set: xr::ActionSet,
    pub action_spaces: Vec<Space>,
    pub action_ipd_inc: xr::Action<bool>,
    pub action_ipd_dec: xr::Action<bool>,
    pub action_roomscale_inc: xr::Action<bool>,
    pub action_roomscale_dec: xr::Action<bool>,
    prev_joy_right_x: f32,
    pub joy_right_x: xr::Action<f32>,
    pub move_x: xr::Action<f32>,
    pub move_y: xr::Action<f32>,
    pub snapturn_dir: f32,
    pub user_hand_right: xr::Path,
    pub user_hand_left: xr::Path,
}

impl OpenXRInput {
    pub fn new(instance: &xr::Instance, session: &xr::Session<xr::OpenGL>) -> Self {
        let action_set = instance.create_action_set("default", "Default action set", 0).unwrap();

        let user_hand_right = instance.string_to_path("/user/hand/right").unwrap();
        let user_hand_left = instance.string_to_path("/user/hand/left").unwrap();

        let move_x = action_set.create_action::<f32>(
            "move_x",
            "Strafe",
            &[user_hand_left]
        ).unwrap();

        let move_y = action_set.create_action::<f32>(
            "move_y",
            "Move forward/backwards",
            &[user_hand_left]
        ).unwrap();

        let joy_right_x = action_set.create_action::<f32>(
            "joy_right_x",
            "Right Joystick X",
            &[
                user_hand_right
            ]
        ).unwrap();

        let action_left_hand = action_set.create_action::<xr::Posef>(
            "left-hand", "Left hand",
            &[
                user_hand_left
            ]
        ).unwrap();

        let action_right_hand = action_set.create_action::<xr::Posef>(
            "right-hand", "Right hand",
            &[
                user_hand_right
            ]
        ).unwrap();

        let action_ipd_inc = action_set.create_action::<bool>(
            "ipd-inc-scale", "Increase IPD scale",
            &[user_hand_right]
        ).unwrap();

        let action_ipd_dec = action_set.create_action::<bool>(
            "ipd-dec-scale", "Decrease IPD scale",
            &[user_hand_left]
        ).unwrap();

        let action_roomscale_inc = action_set.create_action::<bool>(
            "roomscale-inc-scale", "Increase roomscale scale",
            &[user_hand_right]
        ).unwrap();

        let action_roomscale_dec = action_set.create_action::<bool>(
            "roomscale-dec-scale", "Decrease roomscale scale",
            &[user_hand_left]
        ).unwrap();

        instance.suggest_interaction_profile_bindings(
            instance.string_to_path("/interaction_profiles/oculus/touch_controller").unwrap(),
            &[
                xr::Binding::new(&action_left_hand, instance.string_to_path("/user/hand/left/input/grip/pose").unwrap()),
                xr::Binding::new(&action_right_hand, instance.string_to_path("/user/hand/right/input/grip/pose").unwrap()),
                xr::Binding::new(&joy_right_x, instance.string_to_path("/user/hand/right/input/thumbstick/x").unwrap()),
                xr::Binding::new(&move_x, instance.string_to_path("/user/hand/left/input/thumbstick/x").unwrap()),
                xr::Binding::new(&move_y, instance.string_to_path("/user/hand/left/input/thumbstick/y").unwrap()),
                xr::Binding::new(&action_ipd_dec, instance.string_to_path("/user/hand/left/input/y/click").unwrap()),
                xr::Binding::new(&action_ipd_inc, instance.string_to_path("/user/hand/right/input/b/click").unwrap()),
                xr::Binding::new(&action_roomscale_dec, instance.string_to_path("/user/hand/left/input/x/click").unwrap()),
                xr::Binding::new(&action_roomscale_inc, instance.string_to_path("/user/hand/right/input/a/click").unwrap())
            ]
        ).unwrap();

        let action_spaces = vec![
            action_right_hand.create_space(
                session.clone(),
                user_hand_right,
                xr::Posef::IDENTITY
            ).unwrap(),
            action_left_hand.create_space(
                session.clone(),
                user_hand_left,
                xr::Posef::IDENTITY
            ).unwrap()
        ];

        Self {
            action_set,
            action_spaces,
            action_ipd_inc,
            action_ipd_dec,
            action_roomscale_inc,
            action_roomscale_dec,
            prev_joy_right_x: 0.0,
            joy_right_x,
            move_x,
            move_y,
            snapturn_dir: 0.0,
            user_hand_right,
            user_hand_left,
        }
    }

    pub fn poll_actions(&mut self, session: &xr::Session<xr::OpenGL>, predicted_time: xr::Time) {
        session.sync_actions(&[
            xr::ActiveActionSet::new(&self.action_set)
        ]).unwrap();

        let right_x = self.joy_right_x.state(session, self.user_hand_right).unwrap().current_state;

        self.snapturn_dir = 0.0;

        if right_x > 0.70 && self.prev_joy_right_x < 0.65 {
            self.snapturn_dir = 1.0;
        } else if right_x < -0.70 && self.prev_joy_right_x > -0.65 {
            self.snapturn_dir = -1.0;
        }

        println!("prev {}, cur {} = {}", right_x, self.prev_joy_right_x, self.snapturn_dir);

        self.prev_joy_right_x = right_x;

        //let left_hand_location = self.action_spaces[0].locate(&self.reference_space, predicted_time).unwrap();
    }
    
    pub fn get_move(&self, session: &xr::Session<xr::OpenGL>) -> (f32, f32) {
        let x = self.move_x.state(session, self.user_hand_left).unwrap().current_state;
        let y = self.move_y.state(session, self.user_hand_left).unwrap().current_state;

        (x, y)
    }

}
use notify_rust::Notification;
use rand::seq::SliceRandom;

const WELLNESS_TIPS: &[&str] = &[
    "Stand up and walk around your office for 2-3 minutes.",
    "Drink a glass of water to stay hydrated.",
    "Do 10 shoulder rolls to release tension.",
    "Look at something far away for 20 seconds to rest your eyes.",
    "Take 5 deep breaths to reduce stress and increase oxygen flow.",
    "Stretch your arms above your head and hold for 10 seconds.",
    "Do 10 neck stretches - gently tilt your head side to side.",
    "Stand up and do 10 squats to get your blood flowing.",
    "Roll your wrists and ankles to improve circulation.",
    "Walk to get a healthy snack or refill your water bottle.",
    "Stretch your back by doing a seated twist in your chair.",
    "Stand up and shake out your arms and legs.",
    "Close your eyes and relax your facial muscles for 30 seconds.",
    "Open a window or step outside for fresh air.",
    "Massage your temples to relieve tension headaches.",
    "Straighten your posture and adjust your chair height.",
    "Do 10 arm circles forward and backward.",
];

/// Send a break reminder notification with a random wellness tip
///
/// # Arguments
/// * `notification_sound` - Optional sound to play with the notification
/// * `custom_message` - Optional custom message to display instead of a random tip
pub fn send_break_reminder(
    notification_sound: Option<String>,
    custom_message: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = if let Some(message) = custom_message {
        message
    } else {
        WELLNESS_TIPS
            .choose(&mut rand::thread_rng())
            .expect("WELLNESS_TIPS is not empty")
    };

    let mut notification = Notification::new();
    notification
        .summary("Time for a Break!")
        .body(body)
        .timeout(5000); // 5 seconds

    if let Some(sound) = notification_sound {
        notification.sound_name(&sound);
    }

    notification.show()?;
    Ok(())
}

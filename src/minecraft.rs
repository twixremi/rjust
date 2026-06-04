use jni::objects::JValue;
use crate::jni_bridge;

pub struct Minecraft;

impl Minecraft {
    /// Gets the current player's coordinates.
    pub fn get_player_pos() -> anyhow::Result<(f64, f64, f64)> {
        jni_bridge::with_env(|env| {
            // 1. Get Minecraft instance
            let mc_class = env.find_class("net/minecraft/client/Minecraft")?;
            let mc_instance = env.call_static_method(
                mc_class,
                "getInstance",
                "()Lnet/minecraft/client/Minecraft;",
                &[],
            )?.l()?;

            // 2. Get the player
            let player = env.get_field(&mc_instance, "f_91074_", "Lnet/minecraft/client/player/LocalPlayer;")
                .or_else(|_| env.get_field(&mc_instance, "player", "Lnet/minecraft/client/player/LocalPlayer;"))?
                .l()?;

            if player.is_null() {
                return Err(anyhow::anyhow!("Player not in world"));
            }

            // 3. Get X, Y, Z
            let x = env.get_field(&player, "f_19854_", "D")?.d()?; // x
            let y = env.get_field(&player, "f_19855_", "D")?.d()?; // y
            let z = env.get_field(&player, "f_19856_", "D")?.d()?; // z

            Ok((x, y, z))
        })?
    }

    /// Displays a message in the game chat.
    pub fn send_chat_message(message: &str) -> anyhow::Result<()> {
        jni_bridge::with_env(|env| {
            let mc_class = env.find_class("net/minecraft/client/Minecraft")?;
            let mc_instance = env.call_static_method(mc_class, "getInstance", "()Lnet/minecraft/client/Minecraft;", &[])?.l()?;
            let player = env.get_field(&mc_instance, "f_91074_", "Lnet/minecraft/client/player/LocalPlayer;")
                .or_else(|_| env.get_field(&mc_instance, "player", "Lnet/minecraft/client/player/LocalPlayer;"))?
                .l()?;

            if player.is_null() { return Err(anyhow::anyhow!("Player null")); }

            let j_msg = env.new_string(message)?;
            let component = env.call_static_method(
                "net/minecraft/network/chat/Component",
                "m_237113_", // literal
                "(Ljava/lang/String;)Lnet/minecraft/network/chat/MutableComponent;",
                &[JValue::from(&j_msg)],
            ).or_else(|_| {
                env.call_static_method("net/minecraft/network/chat/Component", "literal", "(Ljava/lang/String;)Lnet/minecraft/network/chat/MutableComponent;", &[JValue::from(&j_msg)])
            })?.l()?;

            env.call_method(&player, "m_213846_", "(Lnet/minecraft/network/chat/Component;Z)V", &[JValue::from(&component), JValue::from(false)])
                .or_else(|_| env.call_method(&player, "displayClientMessage", "(Lnet/minecraft/network/chat/Component;Z)V", &[JValue::from(&component), JValue::from(false)]))?;

            Ok(())
        })?
    }
}

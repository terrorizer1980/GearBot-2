use crate::core::BotContext;

impl BotContext {
    // pub fn log(&self, guild_id: GuildId, log: LogType) -> Result<(), Error> {
    //     match self
    //         .log_pumps
    //         .read()
    //         .expect("Log pump list got poisoned!")
    //         .get(&guild_id)
    //     {
    //         Some(pump) => {
    //             pump.send((Utc::now(), log)).map_err(|_| Error::LogError(guild_id))?;
    //             Ok(())
    //         }
    //         None => Ok(()),
    //     }
    // }
}

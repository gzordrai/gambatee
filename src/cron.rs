use std::sync::Arc;

use serenity::all::{CreateEmbed, CreateMessage, Http};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

use crate::{
    config::Stats,
    error::Result,
    voice_stats::{StatsPeriod, UserStats, VoiceStats},
};

pub async fn setup_scheduler(
    http: Arc<Http>,
    settings: Stats,
    voice_stats: Arc<VoiceStats>,
) -> Result<JobScheduler> {
    let scheduler = JobScheduler::new().await?;

    scheduler
        .add(
            create_stats_job(
                http.clone(),
                settings.clone(),
                voice_stats.clone(),
                StatsPeriod::Weekly,
            )
            .await?,
        )
        .await?;

    scheduler
        .add(create_stats_job(http, settings, voice_stats, StatsPeriod::Monthly).await?)
        .await?;

    Ok(scheduler)
}

async fn create_stats_job(
    http: Arc<Http>,
    settings: Stats,
    voice_stats: Arc<VoiceStats>,
    period: StatsPeriod,
) -> Result<Job> {
    let (schedule, title, log_prefix) = match period {
        StatsPeriod::Weekly => (
            settings.weekly_schedule,
            "Les puants de la semaine",
            "weekly",
        ),
        StatsPeriod::Monthly => (settings.monthly_schedule, "Les puants du mois", "monthly"),
    };

    let job = Job::new_async(schedule, move |_, _| {
        let http = http.clone();
        let voice_stats = voice_stats.clone();
        let title = title.to_string();
        let log_prefix = log_prefix.to_string();

        Box::pin(async move {
            let stats_result = voice_stats.get_stats(period, 10).await;

            match stats_result {
                Ok(users) => {
                    let embed = create_stats_embed(&title, &users);
                    let builder = CreateMessage::new().embed(embed);

                    if let Err(e) = settings.channel_id.send_message(&http, builder).await {
                        error!("Failed to send {} stats: {}", log_prefix, e);
                    } else {
                        info!("{} stats sent successfully", log_prefix);
                    }
                }
                Err(e) => {
                    error!("Failed to fetch {} stats: {}", log_prefix, e);
                }
            }
        })
    })?;

    Ok(job)
}

fn create_stats_embed(title: &str, users: &[UserStats]) -> CreateEmbed {
    let mut embed = CreateEmbed::new().title(title).color(0x5865F2).image("https://cdn.ronalbathrooms.com/assets_thumbnails/Magazine/2021/19008/image-thumb__19008__magazine-details-hero-img/378_high.avif");

    for (i, stat) in users.iter().enumerate() {
        embed = embed.field(
            format!("{} - {}", i + 1, stat.username.clone()),
            format!(
                "{:.1}h - {} sessions ({:.1}h avg)",
                stat.total_hours, stat.total_sessions, stat.avg_hours_per_session
            ),
            false,
        );
    }

    if users.is_empty() {
        embed = embed.description("Aucune statistique disponible");
    }

    embed
}

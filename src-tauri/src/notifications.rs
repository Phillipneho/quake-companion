//! Notifications — aggregation of Discord, calendar, and GitHub notifications.
//!
//! Each source is polled independently and merged into a unified feed.
//! Notifications are colour-tagged by source and sorted by timestamp.
//!
//! Sources:
//! - Discord: mentions/DMs via Discord bot token (optional)
//! - GitHub: notifications via GitHub API (optional)
//! - Calendar: upcoming events via Google Calendar API (future)
//!
//! For now, the system supports GitHub (easiest — just needs a PAT)
//! and a simple in-memory store for manual/test notifications.

use serde::{Deserialize, Serialize};

// ---- Notification type -----------------------------------------------------

/// Source of a notification — determines colour tag in the UI.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum NotificationSource {
    Discord,
    GitHub,
    Calendar,
    System,
}

impl NotificationSource {
    /// Colour tag for the UI (hex string).
    pub fn color(self) -> &'static str {
        match self {
            Self::Discord => "#5865F2",  // Discord blurple
            Self::GitHub => "#f78166",   // GitHub orange
            Self::Calendar => "#3a8b9e", // QUAKE teal
            Self::System => "#00d9ff",   // QUAKE cyan
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Discord => "Discord",
            Self::GitHub => "GitHub",
            Self::Calendar => "Calendar",
            Self::System => "System",
        }
    }
}

/// Priority level for visual hierarchy.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    #[default]
    Normal,
    High,
    Urgent,
}

/// A single notification in the unified feed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notification {
    /// Unique id (source-specific, e.g. GitHub notification id).
    pub id: String,
    /// Source platform.
    pub source: NotificationSource,
    /// Title / summary (one line).
    pub title: String,
    /// Body / detail (optional, may be truncated in UI).
    #[serde(default)]
    pub body: String,
    /// ISO 8601 timestamp.
    pub timestamp: String,
    /// Priority level.
    #[serde(default)]
    pub priority: Priority,
    /// URL to open when the user taps the notification.
    #[serde(default)]
    pub url: Option<String>,
    /// Whether the notification has been read/dismissed.
    #[serde(default)]
    pub read: bool,
    /// Whether the notification is snoozed (hidden temporarily).
    #[serde(default)]
    pub snoozed: bool,
}

// ---- Notification feed -----------------------------------------------------

/// The unified notification feed — all sources merged, sorted by time.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NotificationFeed {
    pub notifications: Vec<Notification>,
    pub last_updated: String,
}

impl NotificationFeed {
    /// Get unread notifications, sorted newest first.
    pub fn unread(&self) -> Vec<&Notification> {
        let mut items: Vec<&Notification> = self.notifications
            .iter()
            .filter(|n| !n.read && !n.snoozed)
            .collect();
        items.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        items
    }

    /// Get all notifications, sorted newest first.
    pub fn all_sorted(&self) -> Vec<&Notification> {
        let mut items: Vec<&Notification> = self.notifications.iter().collect();
        items.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        items
    }

    /// Mark a notification as read by id.
    pub fn mark_read(&mut self, id: &str) {
        if let Some(n) = self.notifications.iter_mut().find(|n| n.id == id) {
            n.read = true;
        }
    }

    /// Snooze a notification by id.
    pub fn snooze(&mut self, id: &str) {
        if let Some(n) = self.notifications.iter_mut().find(|n| n.id == id) {
            n.snoozed = true;
        }
    }

    /// Dismiss (mark read + snooze) a notification by id.
    pub fn dismiss(&mut self, id: &str) {
        if let Some(n) = self.notifications.iter_mut().find(|n| n.id == id) {
            n.read = true;
            n.snoozed = true;
        }
    }

    /// Add a notification (replaces if same id exists).
    pub fn add(&mut self, notification: Notification) {
        if let Some(existing) = self.notifications.iter_mut().find(|n| n.id == notification.id) {
            *existing = notification;
        } else {
            self.notifications.push(notification);
        }
    }

    /// Count of unread, non-snoozed notifications.
    pub fn unread_count(&self) -> usize {
        self.notifications.iter().filter(|n| !n.read && !n.snoozed).count()
    }
}

// ---- GitHub notifications --------------------------------------------------

/// Fetch GitHub notifications using a Personal Access Token.
pub async fn fetch_github_notifications(token: &str) -> Result<Vec<Notification>, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.github.com/notifications")
        .header("Authorization", format!("Bearer {token}"))
        .header("User-Agent", "QUAKE-Companion")
        .send()
        .await
        .map_err(|e| format!("GitHub notifications request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("GitHub notifications failed: {}", resp.status()));
    }

    let items: Vec<GitHubNotification> = resp
        .json()
        .await
        .map_err(|e| format!("GitHub notifications parse failed: {e}"))?;

    Ok(items.into_iter().map(|g| Notification {
        id: format!("github:{}", g.id),
        source: NotificationSource::GitHub,
        title: format!("{}: {}", g.subject.r#type, g.subject.title),
        body: String::new(),
        timestamp: g.updated_at,
        priority: Priority::Normal,
        url: Some(g.subject.url),
        read: g.unread == false,
        snoozed: false,
    }).collect())
}

#[derive(Deserialize)]
struct GitHubNotification {
    id: String,
    unread: bool,
    updated_at: String,
    subject: GitHubSubject,
}

#[derive(Deserialize)]
struct GitHubSubject {
    title: String,
    r#type: String,
    url: String,
}

// ---- Config for notification sources ---------------------------------------

/// Configuration for notification sources, stored in the config file.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// GitHub PAT (Personal Access Token) for notifications.
    #[serde(default)]
    pub github_token: Option<String>,
    /// Discord bot token (future).
    #[serde(default)]
    pub discord_token: Option<String>,
    /// Poll interval in seconds.
    #[serde(default = "default_poll_interval_secs")]
    pub poll_interval_secs: u64,
}

fn default_poll_interval_secs() -> u64 {
    60
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_notification(id: &str, source: NotificationSource, title: &str, ts: &str, read: bool) -> Notification {
        Notification {
            id: id.to_string(),
            source,
            title: title.to_string(),
            body: String::new(),
            timestamp: ts.to_string(),
            priority: Priority::Normal,
            url: None,
            read,
            snoozed: false,
        }
    }

    #[test]
    fn feed_unread_filters_and_sorts() {
        let mut feed = NotificationFeed::default();
        feed.add(make_notification("1", NotificationSource::GitHub, "Old", "2026-01-01T00:00:00Z", false));
        feed.add(make_notification("2", NotificationSource::Discord, "New", "2026-01-02T00:00:00Z", false));
        feed.add(make_notification("3", NotificationSource::GitHub, "Read", "2026-01-03T00:00:00Z", true));

        let unread = feed.unread();
        assert_eq!(unread.len(), 2);
        // Newest first
        assert_eq!(unread[0].id, "2");
        assert_eq!(unread[1].id, "1");
    }

    #[test]
    fn feed_mark_read() {
        let mut feed = NotificationFeed::default();
        feed.add(make_notification("1", NotificationSource::GitHub, "Test", "2026-01-01T00:00:00Z", false));
        feed.mark_read("1");
        assert_eq!(feed.unread_count(), 0);
    }

    #[test]
    fn feed_snooze() {
        let mut feed = NotificationFeed::default();
        feed.add(make_notification("1", NotificationSource::Discord, "Test", "2026-01-01T00:00:00Z", false));
        feed.snooze("1");
        assert_eq!(feed.unread_count(), 0);
    }

    #[test]
    fn feed_dismiss() {
        let mut feed = NotificationFeed::default();
        feed.add(make_notification("1", NotificationSource::GitHub, "Test", "2026-01-01T00:00:00Z", false));
        feed.dismiss("1");
        assert_eq!(feed.unread_count(), 0);
    }

    #[test]
    fn feed_add_replaces_existing() {
        let mut feed = NotificationFeed::default();
        feed.add(make_notification("1", NotificationSource::GitHub, "Original", "2026-01-01T00:00:00Z", false));
        feed.add(make_notification("1", NotificationSource::GitHub, "Updated", "2026-01-02T00:00:00Z", false));
        assert_eq!(feed.notifications.len(), 1);
        assert_eq!(feed.notifications[0].title, "Updated");
    }

    #[test]
    fn source_colors_are_distinct() {
        assert_ne!(NotificationSource::Discord.color(), NotificationSource::GitHub.color());
        assert_ne!(NotificationSource::GitHub.color(), NotificationSource::Calendar.color());
        assert_ne!(NotificationSource::Calendar.color(), NotificationSource::System.color());
    }

    #[test]
    fn source_labels_correct() {
        assert_eq!(NotificationSource::Discord.label(), "Discord");
        assert_eq!(NotificationSource::GitHub.label(), "GitHub");
        assert_eq!(NotificationSource::Calendar.label(), "Calendar");
        assert_eq!(NotificationSource::System.label(), "System");
    }

    #[test]
    fn feed_unread_count() {
        let mut feed = NotificationFeed::default();
        feed.add(make_notification("1", NotificationSource::GitHub, "A", "2026-01-01T00:00:00Z", false));
        feed.add(make_notification("2", NotificationSource::GitHub, "B", "2026-01-02T00:00:00Z", false));
        feed.add(make_notification("3", NotificationSource::GitHub, "C", "2026-01-03T00:00:00Z", true));
        assert_eq!(feed.unread_count(), 2);
    }
}
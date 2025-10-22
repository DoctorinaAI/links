#[cfg(test)]
mod tests {
    use crate::{database::create_database, models::ShortLink, services::ShortLinkService};
    use chrono::Utc;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_create_and_resolve_short_link() {
        // Create in-memory SQLite database
        let db = create_database("sqlite::memory:", None)
            .await
            .expect("Failed to create database");

        // Run migrations
        db.migrate().await.expect("Failed to run migrations");

        // Create service
        let service = ShortLinkService::new(db);

        // Create a short link
        let mut params = HashMap::new();
        params.insert("url".to_string(), "https://example.com".to_string());
        params.insert("title".to_string(), "Example".to_string());

        let short_link = ShortLink {
            slug: "test123".to_string(),
            params,
            author: "test_user".to_string(),
            redirect: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Test create
        let created = service
            .create_short_link(short_link.clone())
            .await
            .expect("Failed to create short link");
        assert_eq!(created.slug, "test123");

        // Test resolve
        let resolved = service
            .resolve_short_link("test123")
            .await
            .expect("Failed to resolve short link")
            .expect("Short link not found");
        assert_eq!(resolved.slug, "test123");
        assert_eq!(resolved.author, "test_user");
        assert_eq!(resolved.params.get("url").unwrap(), "https://example.com");
    }

    #[tokio::test]
    async fn test_list_short_links() {
        let db = create_database("sqlite::memory:", None)
            .await
            .expect("Failed to create database");
        db.migrate().await.expect("Failed to run migrations");
        let service = ShortLinkService::new(db);

        // Create multiple short links
        for i in 1..=3 {
            let mut params = HashMap::new();
            params.insert("url".to_string(), format!("https://example{}.com", i));

            let short_link = ShortLink {
                slug: format!("test{}", i),
                params,
                author: "test_user".to_string(),
                redirect: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            service
                .create_short_link(short_link)
                .await
                .expect("Failed to create short link");
        }

        // Test list
        let links = service
            .list_short_links()
            .await
            .expect("Failed to list short links");
        assert_eq!(links.len(), 3);
    }

    #[tokio::test]
    async fn test_update_short_link() {
        let db = create_database("sqlite::memory:", None)
            .await
            .expect("Failed to create database");
        db.migrate().await.expect("Failed to run migrations");
        let service = ShortLinkService::new(db);

        // Create a short link
        let mut params = HashMap::new();
        params.insert("url".to_string(), "https://example.com".to_string());

        let short_link = ShortLink {
            slug: "test123".to_string(),
            params: params.clone(),
            author: "test_user".to_string(),
            redirect: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        service
            .create_short_link(short_link)
            .await
            .expect("Failed to create short link");

        // Update the short link
        params.insert("url".to_string(), "https://updated.com".to_string());
        let update = ShortLink {
            slug: "test123".to_string(),
            params,
            author: "updated_user".to_string(),
            redirect: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let updated = service
            .update_short_link("test123", update)
            .await
            .expect("Failed to update short link");
        assert_eq!(updated.params.get("url").unwrap(), "https://updated.com");
        assert_eq!(updated.author, "updated_user");
    }

    #[tokio::test]
    async fn test_delete_short_link() {
        let db = create_database("sqlite::memory:", None)
            .await
            .expect("Failed to create database");
        db.migrate().await.expect("Failed to run migrations");
        let service = ShortLinkService::new(db);

        // Create a short link
        let mut params = HashMap::new();
        params.insert("url".to_string(), "https://example.com".to_string());

        let short_link = ShortLink {
            slug: "test123".to_string(),
            params,
            author: "test_user".to_string(),
            redirect: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        service
            .create_short_link(short_link)
            .await
            .expect("Failed to create short link");

        // Delete the short link
        service
            .delete_short_link("test123")
            .await
            .expect("Failed to delete short link");

        // Verify it's deleted
        let resolved = service
            .resolve_short_link("test123")
            .await
            .expect("Failed to resolve short link");
        assert!(resolved.is_none());
    }

    #[tokio::test]
    async fn test_click_tracking() {
        let db = create_database("sqlite::memory:", None)
            .await
            .expect("Failed to create database");
        db.migrate().await.expect("Failed to run migrations");
        let service = ShortLinkService::new(db);

        // Create a short link
        let mut params = HashMap::new();
        params.insert("url".to_string(), "https://example.com".to_string());

        let short_link = ShortLink {
            slug: "test123".to_string(),
            params,
            author: "test_user".to_string(),
            redirect: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        service
            .create_short_link(short_link)
            .await
            .expect("Failed to create short link");

        // Record clicks
        for _ in 0..5 {
            service
                .click_short_link("test123")
                .await
                .expect("Failed to record click");
        }

        // Verify click count
        let count = service
            .get_click_count("test123")
            .await
            .expect("Failed to get click count");
        assert_eq!(count, 5);

        // Verify click stats
        let stats = service
            .get_click_stats("test123", None)
            .await
            .expect("Failed to get click stats");
        assert_eq!(stats.len(), 5);
    }
}

use sqlx::{PgPool, Row};
use chrono::{Utc, DateTime};

use super::models::{DbUser, CreateUser, DbTemplate, CreateTemplate, DbTemplateField, CreateTemplateField, CreateSubmitter, DbSubmitter, DbPaymentRecord, CreatePaymentRecord, DbSignatureData, DbSubscriptionPlan, DbTemplateFolder, CreateTemplateFolder, DbSubmissionField, CreateSubmissionField};
use crate::models::role::Role;

// Structured query implementations for better organization
pub struct UserQueries;
pub struct TemplateQueries;
pub struct TemplateFolderQueries;
pub struct TemplateFieldQueries;
pub struct SubmitterQueries;
pub struct SubmissionFieldQueries;

impl UserQueries {
    pub async fn get_user_by_id(pool: &PgPool, id: i64) -> Result<Option<DbUser>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, name, email, password_hash, role, is_active, activation_token, subscription_status, subscription_expires_at, free_usage_count, signature, initials, created_at, updated_at FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => Ok(Some(DbUser {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                email: row.try_get("email")?,
                password_hash: row.try_get("password_hash")?,
                role: row.try_get("role")?,
                is_active: row.try_get("is_active")?,
                activation_token: row.try_get("activation_token")?,
                subscription_status: row.try_get("subscription_status")?,
                subscription_expires_at: row.try_get("subscription_expires_at")?,
                free_usage_count: row.try_get("free_usage_count")?,
                signature: row.try_get("signature")?,
                initials: row.try_get("initials")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            })),
            None => Ok(None),
        }
    }

    pub async fn create_user(pool: &PgPool, user_data: CreateUser) -> Result<DbUser, sqlx::Error> {
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO users (name, email, password_hash, role, is_active, activation_token, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, name, email, password_hash, role, is_active, activation_token, subscription_status, 
                     subscription_expires_at, free_usage_count, signature, initials, created_at, updated_at
            "#
        )
        .bind(&user_data.name)
        .bind(&user_data.email)
        .bind(&user_data.password_hash)
        .bind(&user_data.role)
        .bind(user_data.is_active)
        .bind(&user_data.activation_token)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await?;

        Ok(DbUser {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            email: row.try_get("email")?,
            password_hash: row.try_get("password_hash")?,
            role: row.try_get("role")?,
            is_active: row.try_get("is_active")?,
            activation_token: row.try_get("activation_token")?,
            subscription_status: row.try_get("subscription_status")?,
            subscription_expires_at: row.try_get("subscription_expires_at")?,
            free_usage_count: row.try_get("free_usage_count")?,
            signature: row.try_get("signature")?,
            initials: row.try_get("initials")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<DbUser>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, name, email, password_hash, role, is_active, activation_token, subscription_status, subscription_expires_at, free_usage_count, signature, initials, created_at, updated_at FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => Ok(Some(DbUser {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                email: row.try_get("email")?,
                password_hash: row.try_get("password_hash")?,
                role: row.try_get("role")?,
                is_active: row.try_get("is_active")?,
                activation_token: row.try_get("activation_token")?,
                subscription_status: row.try_get("subscription_status")?,
                subscription_expires_at: row.try_get("subscription_expires_at")?,
                free_usage_count: row.try_get("free_usage_count")?,
                signature: row.try_get("signature")?,
                initials: row.try_get("initials")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            })),
            None => Ok(None),
        }
    }

    pub async fn activate_user(pool: &PgPool, email: String) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE users SET is_active = TRUE, activation_token = NULL WHERE email = $1"
        )
        .bind(email)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update_user_password(pool: &PgPool, user_id: i64, new_password_hash: String) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE users SET password_hash = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(new_password_hash)
        .bind(Utc::now())
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update_user_name(pool: &PgPool, user_id: i64, new_name: String) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE users SET name = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(new_name)
        .bind(Utc::now())
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update_user_email(pool: &PgPool, user_id: i64, new_email: String) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE users SET email = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(new_email)
        .bind(Utc::now())
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update_user_signature(pool: &PgPool, user_id: i64, signature: String) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE users SET signature = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(signature)
        .bind(Utc::now())
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update_user_initials(pool: &PgPool, user_id: i64, initials: String) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE users SET initials = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(initials)
        .bind(Utc::now())
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

}

impl TemplateQueries {
    pub async fn create_template(pool: &PgPool, template_data: CreateTemplate) -> Result<DbTemplate, sqlx::Error> {
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO templates (name, slug, user_id, folder_id, documents, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, name, slug, user_id, folder_id, documents, created_at, updated_at
            "#
        )
        .bind(&template_data.name)
        .bind(&template_data.slug)
        .bind(&template_data.user_id)
        .bind(&template_data.folder_id)
        .bind(&template_data.documents)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await?;

        Ok(DbTemplate {
            id: row.get(0),
            name: row.get(1),
            slug: row.get(2),
            user_id: row.get(3),
            folder_id: row.get(4),
            // fields: None, // Removed - now stored in template_fields table
            documents: row.get(5),
            created_at: row.get(6),
            updated_at: row.get(7),
        })
    }

    pub async fn get_template_by_id(pool: &PgPool, id: i64) -> Result<Option<DbTemplate>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, name, slug, user_id, folder_id, documents, created_at, updated_at FROM templates WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => Ok(Some(DbTemplate {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                slug: row.try_get("slug")?,
                user_id: row.try_get("user_id")?,
                folder_id: row.try_get("folder_id")?,
                // fields: None, // Removed - now stored in template_fields table
                documents: row.try_get("documents")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            })),
            None => Ok(None),
        }
    }

    pub async fn get_template_by_slug(pool: &PgPool, slug: &str) -> Result<Option<DbTemplate>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, name, slug, user_id, folder_id, documents, created_at, updated_at FROM templates WHERE slug = $1"
        )
        .bind(slug)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => Ok(Some(DbTemplate {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                slug: row.try_get("slug")?,
                user_id: row.try_get("user_id")?,
                folder_id: row.try_get("folder_id")?,
                // fields: None, // Removed - now stored in template_fields table
                documents: row.try_get("documents")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            })),
            None => Ok(None),
        }
    }

    pub async fn get_all_templates(pool: &PgPool, user_id: i64) -> Result<Vec<DbTemplate>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, name, slug, user_id, folder_id, documents, created_at, updated_at FROM templates WHERE user_id = $1 AND folder_id IS NULL ORDER BY created_at DESC "
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let mut templates = Vec::new();
        for row in rows {
            templates.push(DbTemplate {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                slug: row.try_get("slug")?,
                user_id: row.try_get("user_id")?,
                folder_id: row.try_get("folder_id")?,
                // fields: None, // Removed - now stored in template_fields table
                documents: row.try_get("documents")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            });
        }
        Ok(templates)
    }

    // Get templates accessible by team (invited users can see inviter's templates)
    pub async fn get_team_templates(pool: &PgPool, user_id: i64) -> Result<Vec<DbTemplate>, sqlx::Error> {
        // Get the user's invitation info to find their team
        let team_query = sqlx::query(
            r#"
            SELECT invited_by_user_id FROM user_invitations 
            WHERE email = (SELECT email FROM users WHERE id = $1) AND is_used = TRUE
            "#
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        // Determine team members
        let team_member_ids = if let Some(row) = team_query {
            // User was invited - can see inviter's templates and own templates
            let invited_by: Option<i64> = row.try_get("invited_by_user_id")?;
            if let Some(inviter_id) = invited_by {
                // Get all users invited by the same inviter (team members)
                let team_rows = sqlx::query(
                    r#"
                    SELECT u.id FROM users u
                    INNER JOIN user_invitations ui ON u.email = ui.email
                    WHERE ui.invited_by_user_id = $1 AND ui.is_used = TRUE
                    UNION
                    SELECT $1 as id
                    "#
                )
                .bind(inviter_id)
                .fetch_all(pool)
                .await?;

                let mut ids: Vec<i64> = team_rows.iter()
                    .filter_map(|row| row.try_get::<i64, _>("id").ok())
                    .collect();
                ids.push(user_id); // Include current user
                ids
            } else {
                vec![user_id] // No inviter, only own templates
            }
        } else {
            // User is admin/inviter - can see own templates + invited users' templates
            let invited_rows = sqlx::query(
                r#"
                SELECT u.id FROM users u
                INNER JOIN user_invitations ui ON u.email = ui.email
                WHERE ui.invited_by_user_id = $1 AND ui.is_used = TRUE
                "#
            )
            .bind(user_id)
            .fetch_all(pool)
            .await?;

            let mut ids: Vec<i64> = invited_rows.iter()
                .filter_map(|row| row.try_get::<i64, _>("id").ok())
                .collect();
            ids.push(user_id); // Include current user (admin)
            ids
        };

        // Get templates for all team members
        if team_member_ids.is_empty() {
            return Ok(vec![]);
        }

        let placeholders: Vec<String> = (1..=team_member_ids.len())
            .map(|i| format!("${}", i))
            .collect();
        let query_str = format!(
            "SELECT id, name, slug, user_id, folder_id, documents, created_at, updated_at 
             FROM templates 
             WHERE user_id IN ({}) AND folder_id IS NULL 
             ORDER BY created_at DESC",
            placeholders.join(", ")
        );

        let mut query = sqlx::query(&query_str);
        for id in team_member_ids {
            query = query.bind(id);
        }

        let rows = query.fetch_all(pool).await?;

        let mut templates = Vec::new();
        for row in rows {
            templates.push(DbTemplate {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                slug: row.try_get("slug")?,
                user_id: row.try_get("user_id")?,
                folder_id: row.try_get("folder_id")?,
                // fields: None, // Removed - now stored in template_fields table
                documents: row.try_get("documents")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            });
        }
        Ok(templates)
    }

    pub async fn update_template(pool: &PgPool, id: i64, name: Option<&str>) -> Result<Option<DbTemplate>, sqlx::Error> {
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            UPDATE templates
            SET name = COALESCE($2, name), updated_at = $3
            WHERE id = $1
            RETURNING id, name, slug, user_id, folder_id, documents, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(name)
        .bind(now)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => Ok(Some(DbTemplate {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                slug: row.try_get("slug")?,
                user_id: row.try_get("user_id")?,
                folder_id: row.try_get("folder_id")?,
                // fields: None, // Removed - now stored in template_fields table
                documents: row.try_get("documents")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            })),
            None => Ok(None),
        }
    }

    pub async fn delete_template(pool: &PgPool, id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM templates WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn clone_template(pool: &PgPool, original_id: i64, user_id: i64, new_name: &str, new_slug: &str) -> Result<Option<DbTemplate>, sqlx::Error> {
        // First get the original template
        if let Some(original) = Self::get_template_by_id(pool, original_id).await? {
            let now = Utc::now();
            let create_data = CreateTemplate {
                name: new_name.to_string(),
                slug: new_slug.to_string(),
                user_id: user_id,
                folder_id: original.folder_id,
                // fields: None, // Removed - will be cloned separately via TemplateFieldQueries
                documents: original.documents,
            };

            Self::create_template(pool, create_data).await.map(Some)
        } else {
            Ok(None)
        }
    }
}

impl TemplateFolderQueries {
    pub async fn create_folder(pool: &PgPool, folder_data: CreateTemplateFolder) -> Result<DbTemplateFolder, sqlx::Error> {
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO template_folders (name, user_id, parent_folder_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, user_id, parent_folder_id, created_at, updated_at
            "#
        )
        .bind(&folder_data.name)
        .bind(folder_data.user_id)
        .bind(folder_data.parent_folder_id)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await?;

        Ok(DbTemplateFolder {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            user_id: row.try_get("user_id")?,
            parent_folder_id: row.try_get("parent_folder_id")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    pub async fn get_folder_by_id(pool: &PgPool, id: i64) -> Result<Option<DbTemplateFolder>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, name, user_id, parent_folder_id, created_at, updated_at FROM template_folders WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => Ok(Some(DbTemplateFolder {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                user_id: row.try_get("user_id")?,
                parent_folder_id: row.try_get("parent_folder_id")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            })),
            None => Ok(None),
        }
    }

    pub async fn get_folders_by_user(pool: &PgPool, user_id: i64) -> Result<Vec<DbTemplateFolder>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, name, user_id, parent_folder_id, created_at, updated_at FROM template_folders WHERE user_id = $1 ORDER BY name ASC"
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let mut folders = Vec::new();
        for row in rows {
            folders.push(DbTemplateFolder {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                user_id: row.try_get("user_id")?,
                parent_folder_id: row.try_get("parent_folder_id")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            });
        }
        Ok(folders)
    }

    pub async fn get_folders_by_parent(pool: &PgPool, user_id: i64, parent_id: Option<i64>) -> Result<Vec<DbTemplateFolder>, sqlx::Error> {
        let rows = if let Some(parent_id) = parent_id {
            sqlx::query(
                "SELECT id, name, user_id, parent_folder_id, created_at, updated_at FROM template_folders WHERE user_id = $1 AND parent_folder_id = $2 ORDER BY name ASC"
            )
            .bind(user_id)
            .bind(parent_id)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query(
                "SELECT id, name, user_id, parent_folder_id, created_at, updated_at FROM template_folders WHERE user_id = $1 AND parent_folder_id IS NULL ORDER BY name ASC"
            )
            .bind(user_id)
            .fetch_all(pool)
            .await?
        };

        let mut folders = Vec::new();
        for row in rows {
            folders.push(DbTemplateFolder {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                user_id: row.try_get("user_id")?,
                parent_folder_id: row.try_get("parent_folder_id")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            });
        }
        Ok(folders)
    }

    pub async fn get_templates_in_folder(pool: &PgPool, user_id: i64, folder_id: Option<i64>) -> Result<Vec<DbTemplate>, sqlx::Error> {
        let rows = if let Some(folder_id) = folder_id {
            sqlx::query(
                "SELECT id, name, slug, user_id, folder_id, documents, created_at, updated_at FROM templates WHERE user_id = $1 AND folder_id = $2 ORDER BY created_at DESC"
            )
            .bind(user_id)
            .bind(folder_id)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query(
                "SELECT id, name, slug, user_id, folder_id, documents, created_at, updated_at FROM templates WHERE user_id = $1 AND folder_id IS NULL ORDER BY created_at DESC"
            )
            .bind(user_id)
            .fetch_all(pool)
            .await?
        };

        let mut templates = Vec::new();
        for row in rows {
            templates.push(DbTemplate {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                slug: row.try_get("slug")?,
                user_id: row.try_get("user_id")?,
                folder_id: row.try_get("folder_id")?,
                documents: row.try_get("documents")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            });
        }
        Ok(templates)
    }

    pub async fn update_folder(pool: &PgPool, id: i64, name: Option<&str>, parent_folder_id: Option<Option<i64>>) -> Result<Option<DbTemplateFolder>, sqlx::Error> {
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            UPDATE template_folders
            SET name = COALESCE($2, name),
                parent_folder_id = COALESCE($3, parent_folder_id),
                updated_at = $4
            WHERE id = $1
            RETURNING id, name, user_id, parent_folder_id, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(name)
        .bind(parent_folder_id)
        .bind(now)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => Ok(Some(DbTemplateFolder {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                user_id: row.try_get("user_id")?,
                parent_folder_id: row.try_get("parent_folder_id")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            })),
            None => Ok(None),
        }
    }

    pub async fn delete_folder(pool: &PgPool, id: i64, folder_user_id: i64) -> Result<bool, sqlx::Error> {
        // First, move all templates in this folder to root (no folder)
        sqlx::query("UPDATE templates SET folder_id = NULL WHERE folder_id = $1 AND user_id = $2")
            .bind(id)
            .bind(folder_user_id)
            .execute(pool)
            .await?;

        // Move child folders to parent folder (or root if no parent)
        let parent_folder_id: Option<i64> = sqlx::query_scalar(
            "SELECT parent_folder_id FROM template_folders WHERE id = $1 AND user_id = $2"
        )
        .bind(id)
        .bind(folder_user_id)
        .fetch_optional(pool)
        .await?
        .flatten();

        sqlx::query("UPDATE template_folders SET parent_folder_id = $1 WHERE parent_folder_id = $2 AND user_id = $3")
            .bind(parent_folder_id)
            .bind(id)
            .bind(folder_user_id)
            .execute(pool)
            .await?;

        // Delete the folder
        let result = sqlx::query("DELETE FROM template_folders WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(folder_user_id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn move_template_to_folder(pool: &PgPool, template_id: i64, folder_id: Option<i64>, user_id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE templates SET folder_id = $1 WHERE id = $2 AND user_id = $3"
        )
        .bind(folder_id)
        .bind(template_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_team_folders(pool: &PgPool, user_id: i64) -> Result<Vec<DbTemplateFolder>, sqlx::Error> {
        // Get the user's invitation info to find their team
        let team_query = sqlx::query(
            r#"
            SELECT invited_by_user_id FROM user_invitations 
            WHERE email = (SELECT email FROM users WHERE id = $1) AND is_used = TRUE
            "#
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        // Determine team members
        let team_member_ids = if let Some(row) = team_query {
            // User was invited - can see inviter's folders and own folders
            let invited_by: Option<i64> = row.try_get("invited_by_user_id")?;
            if let Some(inviter_id) = invited_by {
                // Get all users invited by the same inviter (team members)
                let team_rows = sqlx::query(
                    r#"
                    SELECT u.id FROM users u
                    INNER JOIN user_invitations ui ON u.email = ui.email
                    WHERE ui.invited_by_user_id = $1 AND ui.is_used = TRUE
                    UNION
                    SELECT $1 as id
                    "#
                )
                .bind(inviter_id)
                .fetch_all(pool)
                .await?;

                let mut ids: Vec<i64> = team_rows.iter()
                    .filter_map(|row| row.try_get::<i64, _>("id").ok())
                    .collect();
                ids.push(inviter_id);
                ids
            } else {
                vec![user_id]
            }
        } else {
            // User is the inviter - can see own folders and invited users' folders
            let invited_rows = sqlx::query(
                r#"
                SELECT u.id FROM users u
                INNER JOIN user_invitations ui ON u.email = ui.email
                WHERE ui.invited_by_user_id = $1 AND ui.is_used = TRUE
                UNION
                SELECT $1 as id
                "#
            )
            .bind(user_id)
            .fetch_all(pool)
            .await?;

            invited_rows.iter()
                .filter_map(|row| row.try_get::<i64, _>("id").ok())
                .collect()
        };

        // Get folders for all team members
        let placeholders = team_member_ids.iter()
            .enumerate()
            .map(|(i, _)| format!("${}", i + 1))
            .collect::<Vec<_>>()
            .join(",");

        let query_str = format!(
            "SELECT id, name, user_id, parent_folder_id, created_at, updated_at FROM template_folders WHERE user_id IN ({}) ORDER BY name ASC",
            placeholders
        );

        let mut query = sqlx::query(&query_str);
        for id in &team_member_ids {
            query = query.bind(id);
        }

        let rows = query.fetch_all(pool).await?;

        let mut folders = Vec::new();
        for row in rows {
            folders.push(DbTemplateFolder {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                user_id: row.try_get("user_id")?,
                parent_folder_id: row.try_get("parent_folder_id")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            });
        }
        Ok(folders)
    }

    // Get templates in a specific folder that are accessible by team members
    pub async fn get_team_templates_in_folder(pool: &PgPool, user_id: i64, folder_id: i64) -> Result<Vec<DbTemplate>, sqlx::Error> {
        // Get the user's invitation info to find their team
        let team_query = sqlx::query(
            r#"
            SELECT invited_by_user_id FROM user_invitations 
            WHERE email = (SELECT email FROM users WHERE id = $1) AND is_used = TRUE
            "#
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        // Determine team members
        let team_member_ids = if let Some(row) = team_query {
            // User was invited - can see inviter's templates and own templates
            let invited_by: Option<i64> = row.try_get("invited_by_user_id")?;
            if let Some(inviter_id) = invited_by {
                // Get all users invited by the same inviter (team members)
                let team_rows = sqlx::query(
                    r#"
                    SELECT u.id FROM users u
                    INNER JOIN user_invitations ui ON u.email = ui.email
                    WHERE ui.invited_by_user_id = $1 AND ui.is_used = TRUE
                    UNION
                    SELECT $1 as id
                    "#
                )
                .bind(inviter_id)
                .fetch_all(pool)
                .await?;

                let mut ids: Vec<i64> = team_rows.iter()
                    .filter_map(|row| row.try_get::<i64, _>("id").ok())
                    .collect();
                ids.push(user_id); // Include current user
                ids
            } else {
                vec![user_id] // No inviter, only own templates
            }
        } else {
            // User is admin/inviter - can see own templates + invited users' templates
            let invited_rows = sqlx::query(
                r#"
                SELECT u.id FROM users u
                INNER JOIN user_invitations ui ON u.email = ui.email
                WHERE ui.invited_by_user_id = $1 AND ui.is_used = TRUE
                "#
            )
            .bind(user_id)
            .fetch_all(pool)
            .await?;

            let mut ids: Vec<i64> = invited_rows.iter()
                .filter_map(|row| row.try_get::<i64, _>("id").ok())
                .collect();
            ids.push(user_id); // Include current user (admin)
            ids
        };

        // Get templates for all team members in the specific folder
        if team_member_ids.is_empty() {
            return Ok(vec![]);
        }

        let placeholders: Vec<String> = (1..=team_member_ids.len())
            .map(|i| format!("${}", i))
            .collect();
        let query_str = format!(
            "SELECT id, name, slug, user_id, folder_id, documents, created_at, updated_at 
             FROM templates 
             WHERE user_id IN ({}) AND folder_id = ${}
             ORDER BY created_at DESC",
            placeholders.join(", "),
            team_member_ids.len() + 1
        );

        let mut query = sqlx::query(&query_str);
        for id in team_member_ids {
            query = query.bind(id);
        }
        query = query.bind(folder_id);

        let rows = query.fetch_all(pool).await?;

        let mut templates = Vec::new();
        for row in rows {
            templates.push(DbTemplate {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                slug: row.try_get("slug")?,
                user_id: row.try_get("user_id")?,
                folder_id: row.try_get("folder_id")?,
                documents: row.try_get("documents")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            });
        }
        Ok(templates)
    }
}

impl TemplateFieldQueries {
    pub async fn create_template_field(pool: &PgPool, field_data: CreateTemplateField) -> Result<DbTemplateField, sqlx::Error> {
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO template_fields (
                template_id, name, field_type, required, display_order,
                position, options, metadata, partner, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, template_id, name, field_type, required, display_order,
                     position, options, metadata, partner, created_at, updated_at, deleted_at
            "#
        )
        .bind(field_data.template_id)
        .bind(&field_data.name)
        .bind(&field_data.field_type)
        .bind(field_data.required)
        .bind(field_data.display_order)
        .bind(&field_data.position)
        .bind(&field_data.options)
        .bind(&field_data.metadata)
        .bind(&field_data.partner)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await?;

        Ok(DbTemplateField {
            id: row.try_get("id")?,
            template_id: row.try_get("template_id")?,
            name: row.try_get("name")?,
            field_type: row.try_get("field_type")?,
            required: row.try_get("required")?,
            display_order: row.try_get("display_order")?,
            position: row.try_get("position")?,
            options: row.try_get("options")?,
            metadata: row.try_get("metadata")?,
            partner: row.try_get("partner")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            deleted_at: row.try_get("deleted_at")?,
        })
    }

    pub async fn get_template_fields(pool: &PgPool, template_id: i64) -> Result<Vec<DbTemplateField>, sqlx::Error> {
        sqlx::query_as::<_, DbTemplateField>(
            "SELECT * FROM template_fields WHERE template_id = $1 AND deleted_at IS NULL ORDER BY display_order"
        )
        .bind(template_id)
        .fetch_all(pool)
        .await
    }

    pub async fn get_all_template_fields(pool: &PgPool, template_id: i64) -> Result<Vec<DbTemplateField>, sqlx::Error> {
        sqlx::query_as::<_, DbTemplateField>(
            "SELECT * FROM template_fields WHERE template_id = $1 ORDER BY display_order"
        )
        .bind(template_id)
        .fetch_all(pool)
        .await
    }

    pub async fn get_template_field_by_id(pool: &PgPool, field_id: i64) -> Result<Option<DbTemplateField>, sqlx::Error> {
        sqlx::query_as::<_, DbTemplateField>(
            "SELECT * FROM template_fields WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(field_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn update_template_field(pool: &PgPool, field_id: i64, field_data: CreateTemplateField) -> Result<Option<DbTemplateField>, sqlx::Error> {
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            UPDATE template_fields SET
                name = $2, field_type = $3, required = $4, display_order = $5,
                position = $6, options = $7, metadata = $8, partner = $9, updated_at = $10
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, template_id, name, field_type, required, display_order,
                     position, options, metadata, partner, created_at, updated_at, deleted_at
            "#
        )
        .bind(field_id)
        .bind(&field_data.name)
        .bind(&field_data.field_type)
        .bind(field_data.required)
        .bind(field_data.display_order)
        .bind(&field_data.position)
        .bind(&field_data.options)
        .bind(&field_data.metadata)
        .bind(&field_data.partner)
        .bind(now)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => Ok(Some(DbTemplateField {
                id: row.try_get("id")?,
                template_id: row.try_get("template_id")?,
                name: row.try_get("name")?,
                field_type: row.try_get("field_type")?,
                required: row.try_get("required")?,
                display_order: row.try_get("display_order")?,
                position: row.try_get("position")?,
                options: row.try_get("options")?,
                metadata: row.try_get("metadata")?,
                partner: row.try_get("partner")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
                deleted_at: row.try_get("deleted_at")?,
            })),
            None => Ok(None),
        }
    }

    pub async fn delete_template_field(pool: &PgPool, field_id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE template_fields SET deleted_at = CURRENT_TIMESTAMP WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(field_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn clone_template_fields(pool: &PgPool, from_template_id: i64, to_template_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO template_fields (
                template_id, name, field_type, required, display_order,
                position, options, metadata, partner, created_at, updated_at
            )
            SELECT
                $2 as template_id, name, field_type, required, display_order,
                position, options, metadata, partner, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
            FROM template_fields
            WHERE template_id = $1 AND deleted_at IS NULL
            "#
        )
        .bind(from_template_id)
        .bind(to_template_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_template_fields_with_positions(pool: &PgPool, template_id: i64) -> Result<Vec<DbTemplateField>, sqlx::Error> {
        sqlx::query_as::<_, DbTemplateField>(
            "SELECT * FROM template_fields 
             WHERE template_id = $1 AND position IS NOT NULL AND deleted_at IS NULL
             ORDER BY display_order"
        )
        .bind(template_id)
        .fetch_all(pool)
        .await
    }
}

impl SubmitterQueries {
    pub async fn create_submitter(pool: &PgPool, submitter_data: CreateSubmitter) -> Result<DbSubmitter, sqlx::Error> {
        let now = Utc::now();
        eprintln!("Creating submitter: template_id={}, user_id={}, name={}, email={}, token={}",
            submitter_data.template_id, submitter_data.user_id, submitter_data.name, submitter_data.email, submitter_data.token);
        let row = sqlx::query(
            "INSERT INTO submitters (template_id, user_id, name, email, status, signed_at, token, bulk_signatures, ip_address, user_agent, reminder_config, reminder_count, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
             RETURNING id, template_id, user_id, name, email, status, signed_at, token, bulk_signatures, ip_address, user_agent, reminder_config, last_reminder_sent_at, reminder_count, created_at, updated_at "
        )
        .bind(submitter_data.template_id)
        .bind(submitter_data.user_id)
        .bind(submitter_data.name)
        .bind(submitter_data.email)
        .bind(submitter_data.status)
        .bind(None as Option<DateTime<Utc>>)
        .bind(submitter_data.token)
        .bind(None as Option<serde_json::Value>) // bulk_signatures
        .bind(None as Option<String>) // ip_address
        .bind(None as Option<String>) // user_agent
        .bind(submitter_data.reminder_config) // reminder_config
        .bind(0) // reminder_count
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await?;

        eprintln!("Submitter created successfully: id={}", row.get::<i64, _>(0));
        Ok(DbSubmitter {
            id: row.get(0),
            template_id: row.get(1),
            user_id: row.get(2),
            name: row.get(3),
            email: row.get(4),
            status: row.get(5),
            signed_at: row.get(6),
            token: row.get(7),
            bulk_signatures: row.get(8),
            ip_address: row.get(9),
            user_agent: row.get(10),
            reminder_config: row.get(11),
            last_reminder_sent_at: row.get(12),
            reminder_count: row.get(13),
            created_at: row.get(14),
            updated_at: row.get(15),
        })
    }

    pub async fn get_submitters_by_template(pool: &PgPool, template_id: i64) -> Result<Vec<DbSubmitter>, sqlx::Error> {
        eprintln!("Getting submitters for template_id: {}", template_id);
        let rows = sqlx::query(
            "SELECT id, template_id, user_id, name, email, status, signed_at, token, bulk_signatures, ip_address, user_agent, reminder_config, last_reminder_sent_at, reminder_count, created_at, updated_at 
             FROM submitters WHERE template_id = $1 ORDER BY created_at "
        )
        .bind(template_id)
        .fetch_all(pool)
        .await?;

        eprintln!("Found {} submitters", rows.len());
        let mut submitters = Vec::new();
        for row in rows {
            submitters.push(DbSubmitter {
                id: row.get(0),
                template_id: row.get(1),
                user_id: row.get(2),
                name: row.get(3),
                email: row.get(4),
                status: row.get(5),
                signed_at: row.get(6),
                token: row.get(7),
                bulk_signatures: row.get(8),
                ip_address: row.get(9),
                user_agent: row.get(10),
                reminder_config: row.get(11),
                last_reminder_sent_at: row.get(12),
                reminder_count: row.get(13),
                created_at: row.get(14),
                updated_at: row.get(15),
            });
        }
        Ok(submitters)
    }

    pub async fn get_submitters_by_user(pool: &PgPool, user_id: i64) -> Result<Vec<DbSubmitter>, sqlx::Error> {
        eprintln!("Getting submitters for user_id: {}", user_id);
        let rows = sqlx::query(
            "SELECT id, template_id, user_id, name, email, status, signed_at, token, bulk_signatures, ip_address, user_agent, reminder_config, last_reminder_sent_at, reminder_count, created_at, updated_at 
             FROM submitters WHERE user_id = $1 ORDER BY created_at "
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        eprintln!("Found {} submitters", rows.len());
        let mut submitters = Vec::new();
        for row in rows {
            submitters.push(DbSubmitter {
                id: row.get(0),
                template_id: row.get(1),
                user_id: row.get(2),
                name: row.get(3),
                email: row.get(4),
                status: row.get(5),
                signed_at: row.get(6),
                token: row.get(7),
                bulk_signatures: row.get(8),
                ip_address: row.get(9),
                user_agent: row.get(10),
                reminder_config: row.get(11),
                last_reminder_sent_at: row.get(12),
                reminder_count: row.get(13),
                created_at: row.get(14),
                updated_at: row.get(15),
            });
        }
        Ok(submitters)
    }

    pub async fn get_submitter_by_token(pool: &PgPool, token: &str) -> Result<Option<DbSubmitter>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, template_id, user_id, name, email, status, signed_at, token, bulk_signatures, ip_address, user_agent, reminder_config, last_reminder_sent_at, reminder_count, created_at, updated_at 
             FROM submitters WHERE token = $1"
        )
        .bind(token)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(DbSubmitter {
                id: row.get(0),
                template_id: row.get(1),
                user_id: row.get(2),
                name: row.get(3),
                email: row.get(4),
                status: row.get(5),
                signed_at: row.get(6),
                token: row.get(7),
                bulk_signatures: row.get(8),
                ip_address: row.get(9),
                user_agent: row.get(10),
                reminder_config: row.get(11),
                last_reminder_sent_at: row.get(12),
                reminder_count: row.get(13),
                created_at: row.get(14),
                updated_at: row.get(15),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_submitter(pool: &PgPool, id: i64, status: Option<&str>) -> Result<Option<DbSubmitter>, sqlx::Error> {
        let now = Utc::now();
        let signed_at = if status == Some("signed") { Some(now) } else { None };
        
        let row = sqlx::query(
            "UPDATE submitters SET status = COALESCE($1, status), signed_at = COALESCE($2, signed_at), updated_at = $3 
             WHERE id = $4 
             RETURNING id, template_id, user_id, name, email, status, signed_at, token, bulk_signatures, ip_address, user_agent, reminder_config, last_reminder_sent_at, reminder_count, created_at, updated_at "
        )
        .bind(status)
        .bind(signed_at)
        .bind(now)
        .bind(id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(DbSubmitter {
                id: row.get(0),
                template_id: row.get(1),
                user_id: row.get(2),
                name: row.get(3),
                email: row.get(4),
                status: row.get(5),
                signed_at: row.get(6),
                token: row.get(7),
                bulk_signatures: row.get(8),
                ip_address: row.get(9),
                user_agent: row.get(10),
                reminder_config: row.get(11),
                last_reminder_sent_at: row.get(12),
                reminder_count: row.get(13),
                created_at: row.get(14),
                updated_at: row.get(15),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_submitter_with_signatures(
        pool: &PgPool,
        id: i64,
        bulk_signatures: &serde_json::Value,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<Option<DbSubmitter>, sqlx::Error> {
        let now = Utc::now();

        let row = sqlx::query(
            "UPDATE submitters SET bulk_signatures = $1, ip_address = $2, user_agent = $3, status = 'signed', signed_at = $4, updated_at = $4 
             WHERE id = $5 
             RETURNING id, template_id, user_id, name, email, status, signed_at, token, bulk_signatures, ip_address, user_agent, reminder_config, last_reminder_sent_at, reminder_count, created_at, updated_at "
        )
        .bind(bulk_signatures)
        .bind(ip_address)
        .bind(user_agent)
        .bind(now)
        .bind(id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(DbSubmitter {
                id: row.get(0),
                template_id: row.get(1),
                user_id: row.get(2),
                name: row.get(3),
                email: row.get(4),
                status: row.get(5),
                signed_at: row.get(6),
                token: row.get(7),
                bulk_signatures: row.get(8),
                ip_address: row.get(9),
                user_agent: row.get(10),
                reminder_config: row.get(11),
                last_reminder_sent_at: row.get(12),
                reminder_count: row.get(13),
                created_at: row.get(14),
                updated_at: row.get(15),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_submitter_by_id(pool: &PgPool, id: i64) -> Result<Option<DbSubmitter>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, template_id, user_id, name, email, status, signed_at, token, bulk_signatures, ip_address, user_agent, reminder_config, last_reminder_sent_at, reminder_count, created_at, updated_at 
             FROM submitters WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(DbSubmitter {
                id: row.get(0),
                template_id: row.get(1),
                user_id: row.get(2),
                name: row.get(3),
                email: row.get(4),
                status: row.get(5),
                signed_at: row.get(6),
                token: row.get(7),
                bulk_signatures: row.get(8),
                ip_address: row.get(9),
                user_agent: row.get(10),
                reminder_config: row.get(11),
                last_reminder_sent_at: row.get(12),
                reminder_count: row.get(13),
                created_at: row.get(14),
                updated_at: row.get(15),
            }))
        } else {
            Ok(None)
        }
    }

    // Get submitters that need reminder emails
    pub async fn get_pending_reminders(pool: &PgPool) -> Result<Vec<DbSubmitter>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, template_id, user_id, name, email, status, signed_at, token, bulk_signatures, ip_address, user_agent, reminder_config, last_reminder_sent_at, reminder_count, created_at, updated_at
            FROM submitters
            WHERE status IN ('pending', 'sent', 'viewed')
              AND reminder_config IS NOT NULL
              AND reminder_count < 3
            ORDER BY created_at
            "#
        )
        .fetch_all(pool)
        .await?;

        let mut submitters = Vec::new();
        for row in rows {
            submitters.push(DbSubmitter {
                id: row.get(0),
                template_id: row.get(1),
                user_id: row.get(2),
                name: row.get(3),
                email: row.get(4),
                status: row.get(5),
                signed_at: row.get(6),
                token: row.get(7),
                bulk_signatures: row.get(8),
                ip_address: row.get(9),
                user_agent: row.get(10),
                reminder_config: row.get(11),
                last_reminder_sent_at: row.get(12),
                reminder_count: row.get(13),
                created_at: row.get(14),
                updated_at: row.get(15),
            });
        }
        Ok(submitters)
    }

    // Update reminder status after sending
    pub async fn update_reminder_sent(pool: &PgPool, submitter_id: i64) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        sqlx::query(
            "UPDATE submitters 
             SET last_reminder_sent_at = $1, 
                 reminder_count = reminder_count + 1,
                 updated_at = $1
             WHERE id = $2"
        )
        .bind(now)
        .bind(submitter_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn delete_submitter(pool: &PgPool, id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM submitters WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // Get submitters accessible by team (invited users can see inviter's submitters)
    pub async fn get_team_submitters(pool: &PgPool, user_id: i64) -> Result<Vec<DbSubmitter>, sqlx::Error> {
        // Get the user's invitation info to find their team
        let team_query = sqlx::query(
            r#"
            SELECT invited_by_user_id FROM user_invitations 
            WHERE email = (SELECT email FROM users WHERE id = $1) AND is_used = TRUE
            "#
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        // Determine team members
        let team_member_ids = if let Some(row) = team_query {
            // User was invited - can see inviter's submitters and own submitters
            let invited_by: Option<i64> = row.try_get("invited_by_user_id")?;
            if let Some(inviter_id) = invited_by {
                // Get all users invited by the same inviter (team members)
                let team_rows = sqlx::query(
                    r#"
                    SELECT u.id FROM users u
                    INNER JOIN user_invitations ui ON u.email = ui.email
                    WHERE ui.invited_by_user_id = $1 AND ui.is_used = TRUE
                    UNION
                    SELECT $1 as id
                    "#
                )
                .bind(inviter_id)
                .fetch_all(pool)
                .await?;

                let mut ids: Vec<i64> = team_rows.iter()
                    .filter_map(|row| row.try_get::<i64, _>("id").ok())
                    .collect();
                ids.push(user_id); // Include current user
                ids
            } else {
                vec![user_id] // No inviter, only own submitters
            }
        } else {
            // User is admin/inviter - can see own submitters + invited users' submitters
            let invited_rows = sqlx::query(
                r#"
                SELECT u.id FROM users u
                INNER JOIN user_invitations ui ON u.email = ui.email
                WHERE ui.invited_by_user_id = $1 AND ui.is_used = TRUE
                "#
            )
            .bind(user_id)
            .fetch_all(pool)
            .await?;

            let mut ids: Vec<i64> = invited_rows.iter()
                .filter_map(|row| row.try_get::<i64, _>("id").ok())
                .collect();
            ids.push(user_id); // Include current user (admin)
            ids
        };

        // Get submitters for all team members
        if team_member_ids.is_empty() {
            return Ok(vec![]);
        }

        let placeholders: Vec<String> = (1..=team_member_ids.len())
            .map(|i| format!("${}", i))
            .collect();
        let query_str = format!(
            "SELECT id, template_id, user_id, name, email, status, signed_at, token, bulk_signatures, ip_address, user_agent, reminder_config, last_reminder_sent_at, reminder_count, created_at, updated_at 
             FROM submitters 
             WHERE user_id IN ({}) 
             ORDER BY created_at DESC",
            placeholders.join(", ")
        );

        let mut query = sqlx::query(&query_str);
        for id in team_member_ids {
            query = query.bind(id);
        }

        let rows = query.fetch_all(pool).await?;

        let mut submitters = Vec::new();
        for row in rows {
            submitters.push(DbSubmitter {
                id: row.get(0),
                template_id: row.get(1),
                user_id: row.get(2),
                name: row.get(3),
                email: row.get(4),
                status: row.get(5),
                signed_at: row.get(6),
                token: row.get(7),
                bulk_signatures: row.get(8),
                ip_address: row.get(9),
                user_agent: row.get(10),
                reminder_config: row.get(11),
                last_reminder_sent_at: row.get(12),
                reminder_count: row.get(13),
                created_at: row.get(14),
                updated_at: row.get(15),
            });
        }
        Ok(submitters)
    }
}

impl SubmissionFieldQueries {
    pub async fn create_submission_field(pool: &PgPool, field_data: CreateSubmissionField) -> Result<DbSubmissionField, sqlx::Error> {
        let now = Utc::now();
        let row = sqlx::query(
            "INSERT INTO submission_fields (submitter_id, template_field_id, name, field_type, required, display_order, position, options, metadata, partner, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
             RETURNING id, submitter_id, template_field_id, name, field_type, required, display_order, position, options, metadata, partner, created_at, updated_at"
        )
        .bind(field_data.submitter_id)
        .bind(field_data.template_field_id)
        .bind(field_data.name)
        .bind(field_data.field_type)
        .bind(field_data.required)
        .bind(field_data.display_order)
        .bind(field_data.position)
        .bind(field_data.options)
        .bind(field_data.metadata)
        .bind(field_data.partner)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await?;

        Ok(DbSubmissionField {
            id: row.get(0),
            submitter_id: row.get(1),
            template_field_id: row.get(2),
            name: row.get(3),
            field_type: row.get(4),
            required: row.get(5),
            display_order: row.get(6),
            position: row.get(7),
            options: row.get(8),
            metadata: row.get(9),
            partner: row.get(10),
            created_at: row.get(11),
            updated_at: row.get(12),
        })
    }

    pub async fn get_submission_fields_by_submitter_id(pool: &PgPool, submitter_id: i64) -> Result<Vec<DbSubmissionField>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, submitter_id, template_field_id, name, field_type, required, display_order, position, options, metadata, partner, created_at, updated_at
             FROM submission_fields WHERE submitter_id = $1 ORDER BY display_order"
        )
        .bind(submitter_id)
        .fetch_all(pool)
        .await?;

        let mut fields = Vec::new();
        for row in rows {
            fields.push(DbSubmissionField {
                id: row.get(0),
                submitter_id: row.get(1),
                template_field_id: row.get(2),
                name: row.get(3),
                field_type: row.get(4),
                required: row.get(5),
                display_order: row.get(6),
                position: row.get(7),
                options: row.get(8),
                metadata: row.get(9),
                partner: row.get(10),
                created_at: row.get(11),
                updated_at: row.get(12),
            });
        }
        Ok(fields)
    }
}

pub struct SignatureQueries;

impl SignatureQueries {






    // DEPRECATED: This function is not used with bulk signatures
    // pub async fn get_signature_history_by_field(
    //     pool: &PgPool,
    //     submitter_id: i64,
    //     field_name: &str,
    // ) -> Result<Vec<DbSignaturePosition>, sqlx::Error> {
    //     // Now we use bulk_signatures instead
    //     Ok(Vec::new())
    // }

    pub async fn get_submitter_with_signatures(
        pool: &PgPool,
        submitter_id: i64,
    ) -> Result<Option<DbSubmitter>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, template_id, user_id, name, email, status, signed_at, token, bulk_signatures, ip_address, user_agent, reminder_config, last_reminder_sent_at, reminder_count, created_at, updated_at
            FROM submitters
            WHERE id = $1 AND bulk_signatures IS NOT NULL
            "#
        )
        .bind(submitter_id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => Ok(Some(DbSubmitter {
                id: row.get(0),
                template_id: row.get(1),
                user_id: row.get(2),
                name: row.get(3),
                email: row.get(4),
                status: row.get(5),
                signed_at: row.get(6),
                token: row.get(7),
                bulk_signatures: row.get(8),
                ip_address: row.get(9),
                user_agent: row.get(10),
                reminder_config: row.get(11),
                last_reminder_sent_at: row.get(12),
                reminder_count: row.get(13),
                created_at: row.get(14),
                updated_at: row.get(15),
            })),
            None => Ok(None),
        }
    }

    // DEPRECATED: This function is not used with bulk signatures
    // pub async fn get_latest_signature_by_field(
    //     pool: &PgPool,
    //     submitter_id: i64,
    //     field_name: &str,
    // ) -> Result<Option<DbSignaturePosition>, sqlx::Error> {
    //     // Now we use bulk_signatures instead
    //     Ok(None)
    // }

    pub async fn get_signature_data_by_submitter(
        pool: &PgPool,
        submitter_id: i64,
    ) -> Result<Option<DbSignatureData>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, submitter_id, signature_image, signature_value, signed_at, ip_address, user_agent
            FROM signature_data
            WHERE submitter_id = $1
            ORDER BY signed_at DESC
            LIMIT 1
            "#
        )
        .bind(submitter_id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(DbSignatureData {
                id: row.try_get("id")?,
                submitter_id: row.try_get("submitter_id")?,
                signature_value: row.try_get("signature_value")?,
                signed_at: row.try_get("signed_at")?,
                ip_address: row.try_get("ip_address")?,
                user_agent: row.try_get("user_agent")?,
            }))
        } else {
            Ok(None)
        }
    }
}

// User Reminder Settings Queries
pub struct UserReminderSettingsQueries;

impl UserReminderSettingsQueries {
    // Get user reminder settings
    pub async fn get_by_user_id(pool: &PgPool, user_id: i64) -> Result<Option<super::models::DbUserReminderSettings>, sqlx::Error> {
        let row = sqlx::query_as::<_, super::models::DbUserReminderSettings>(
            "SELECT id, user_id, first_reminder_hours, second_reminder_hours, third_reminder_hours, created_at, updated_at 
             FROM user_reminder_settings WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(row)
    }

    // Create default reminder settings for new user
    pub async fn create(pool: &PgPool, settings_data: super::models::CreateUserReminderSettings) -> Result<super::models::DbUserReminderSettings, sqlx::Error> {
        let now = Utc::now();

        let row = sqlx::query_as::<_, super::models::DbUserReminderSettings>(
            r#"
            INSERT INTO user_reminder_settings (user_id, first_reminder_hours, second_reminder_hours, third_reminder_hours, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, first_reminder_hours, second_reminder_hours, third_reminder_hours, created_at, updated_at
            "#
        )
        .bind(settings_data.user_id)
        .bind(settings_data.first_reminder_hours)
        .bind(settings_data.second_reminder_hours)
        .bind(settings_data.third_reminder_hours)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await?;

        Ok(row)
    }

    // Update user reminder settings
    pub async fn update(pool: &PgPool, user_id: i64, update_data: super::models::UpdateUserReminderSettings) -> Result<Option<super::models::DbUserReminderSettings>, sqlx::Error> {
        let now = Utc::now();

        let row = sqlx::query_as::<_, super::models::DbUserReminderSettings>(
            r#"
            UPDATE user_reminder_settings 
            SET first_reminder_hours = COALESCE($1, first_reminder_hours),
                second_reminder_hours = COALESCE($2, second_reminder_hours),
                third_reminder_hours = COALESCE($3, third_reminder_hours),
                updated_at = $4
            WHERE user_id = $5
            RETURNING id, user_id, first_reminder_hours, second_reminder_hours, third_reminder_hours, created_at, updated_at
            "#
        )
        .bind(update_data.first_reminder_hours)
        .bind(update_data.second_reminder_hours)
        .bind(update_data.third_reminder_hours)
        .bind(now)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(row)
    }

    // Create default settings if not exists (helper for when creating submitters)
    pub async fn get_or_create_default(pool: &PgPool, user_id: i64) -> Result<super::models::DbUserReminderSettings, sqlx::Error> {
        // Try to get existing settings
        if let Some(settings) = Self::get_by_user_id(pool, user_id).await? {
            return Ok(settings);
        }

        // Create default settings (all NULL - user must configure)
        let default_settings = super::models::CreateUserReminderSettings {
            user_id,
            first_reminder_hours: None,   // User must set
            second_reminder_hours: None,  // User must set
            third_reminder_hours: None,   // User must set
        };

        Self::create(pool, default_settings).await
    }
}

// Simplified subscription-related queries
pub struct SubscriptionQueries;

impl SubscriptionQueries {
    // Create payment record
    pub async fn create_payment_record(pool: &PgPool, payment_data: CreatePaymentRecord) -> Result<DbPaymentRecord, sqlx::Error> {
        let now = Utc::now();

        let row = sqlx::query_as::<_, DbPaymentRecord>(
            r#"
            INSERT INTO payment_records (user_id, stripe_session_id, amount_cents, currency, status, metadata, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#
        )
        .bind(&payment_data.user_id)
        .bind(&payment_data.stripe_session_id)
        .bind(&payment_data.amount_cents)
        .bind(&payment_data.currency)
        .bind(&payment_data.status)
        .bind(&payment_data.metadata)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await?;

        Ok(row)
    }


    // Update user subscription status sau khi thanh toán thành công
    pub async fn update_user_subscription_status(pool: &PgPool, user_id: i64, status: &str, expires_at: Option<DateTime<Utc>>) -> Result<(), sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            "UPDATE users SET subscription_status = $1, subscription_expires_at = $2, updated_at = $3 WHERE id = $4"
        )
        .bind(status)
        .bind(expires_at)
        .bind(now)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    // Increment user free usage count
    pub async fn increment_user_usage(pool: &PgPool, user_id: i64) -> Result<i32, sqlx::Error> {
        let row = sqlx::query(
            "UPDATE users SET free_usage_count = free_usage_count + 1, updated_at = NOW() WHERE id = $1 RETURNING free_usage_count"
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(row.try_get("free_usage_count")?)
    }

    pub async fn increment_user_usage_by(pool: &PgPool, user_id: i64, count: i32) -> Result<i32, sqlx::Error> {
        let row = sqlx::query(
            "UPDATE users SET free_usage_count = free_usage_count + $2, updated_at = NOW() WHERE id = $1 RETURNING free_usage_count"
        )
        .bind(user_id)
        .bind(count)
        .fetch_one(pool)
        .await?;

        Ok(row.try_get("free_usage_count")?)
    }

    // Get user subscription status
    pub async fn get_user_subscription_status(pool: &PgPool, user_id: i64) -> Result<Option<DbUser>, sqlx::Error> {
        let row = sqlx::query_as::<_, DbUser>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(row)
    }

}

pub struct OAuthTokenQueries;

impl OAuthTokenQueries {
    pub async fn get_oauth_token(pool: &PgPool, user_id: i64, provider: &str) -> Result<Option<super::models::DbOAuthToken>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, user_id, provider, access_token, refresh_token, expires_at, created_at, updated_at FROM oauth_tokens WHERE user_id = $1 AND provider = $2 ORDER BY created_at DESC LIMIT 1"
        )
        .bind(user_id)
        .bind(provider)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => Ok(Some(super::models::DbOAuthToken {
                id: row.try_get("id")?,
                user_id: row.try_get("user_id")?,
                provider: row.try_get("provider")?,
                access_token: row.try_get("access_token")?,
                refresh_token: row.try_get("refresh_token")?,
                expires_at: row.try_get("expires_at")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            })),
            None => Ok(None),
        }
    }

    pub async fn create_oauth_token(pool: &PgPool, token_data: super::models::CreateOAuthToken) -> Result<super::models::DbOAuthToken, sqlx::Error> {
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO oauth_tokens (user_id, provider, access_token, refresh_token, expires_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, user_id, provider, access_token, refresh_token, expires_at, created_at, updated_at
            "#
        )
        .bind(token_data.user_id)
        .bind(&token_data.provider)
        .bind(&token_data.access_token)
        .bind(&token_data.refresh_token)
        .bind(token_data.expires_at)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await?;

        Ok(super::models::DbOAuthToken {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            provider: row.try_get("provider")?,
            access_token: row.try_get("access_token")?,
            refresh_token: row.try_get("refresh_token")?,
            expires_at: row.try_get("expires_at")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    pub async fn update_oauth_token(pool: &PgPool, user_id: i64, provider: &str, access_token: &str, refresh_token: Option<&str>, expires_at: Option<DateTime<Utc>>) -> Result<(), sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            "UPDATE oauth_tokens SET access_token = $1, refresh_token = $2, expires_at = $3, updated_at = $4 WHERE user_id = $5 AND provider = $6"
        )
        .bind(access_token)
        .bind(refresh_token)
        .bind(expires_at)
        .bind(now)
        .bind(user_id)
        .bind(provider)
        .execute(pool)
        .await?;

        Ok(())
    }
}

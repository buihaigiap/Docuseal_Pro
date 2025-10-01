use sqlx::{PgPool, Row};
use chrono::Utc;

use super::models::{DbUser, CreateUser, DbTemplate, CreateTemplate};

pub struct UserQueries;

impl UserQueries {
    pub async fn create_user(pool: &PgPool, user_data: CreateUser) -> Result<DbUser, sqlx::Error> {
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO users (name, email, password_hash, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, email, password_hash, created_at, updated_at
            "#
        )
        .bind(&user_data.name)
        .bind(&user_data.email)
        .bind(&user_data.password_hash)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await?;

        Ok(DbUser {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            email: row.try_get("email")?,
            password_hash: row.try_get("password_hash")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<DbUser>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, name, email, password_hash, created_at, updated_at FROM users WHERE email = $1"
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
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            })),
            None => Ok(None),
        }
    }
}

pub struct TemplateQueries;

impl TemplateQueries {
    pub async fn create_template(pool: &PgPool, template_data: CreateTemplate) -> Result<DbTemplate, sqlx::Error> {
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO templates (name, slug, user_id, fields, submitters, documents, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, name, slug, user_id, fields, submitters, documents, created_at, updated_at
            "#
        )
        .bind(&template_data.name)
        .bind(&template_data.slug)
        .bind(&template_data.user_id)
        .bind(&template_data.fields)
        .bind(&template_data.submitters)
        .bind(&template_data.documents)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await?;

        Ok(DbTemplate {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            slug: row.try_get("slug")?,
            user_id: row.try_get("user_id")?,
            fields: row.try_get("fields")?,
            submitters: row.try_get("submitters")?,
            documents: row.try_get("documents")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    pub async fn get_template_by_id(pool: &PgPool, id: i64) -> Result<Option<DbTemplate>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, name, slug, user_id, fields, submitters, documents, created_at, updated_at FROM templates WHERE id = $1"
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
                fields: row.try_get("fields")?,
                submitters: row.try_get("submitters")?,
                documents: row.try_get("documents")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            })),
            None => Ok(None),
        }
    }

    pub async fn get_template_by_slug(pool: &PgPool, slug: &str) -> Result<Option<DbTemplate>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, name, slug, user_id, fields, submitters, documents, created_at, updated_at FROM templates WHERE slug = $1"
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
                fields: row.try_get("fields")?,
                submitters: row.try_get("submitters")?,
                documents: row.try_get("documents")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            })),
            None => Ok(None),
        }
    }

    pub async fn get_all_templates(pool: &PgPool, user_id: i64) -> Result<Vec<DbTemplate>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, name, slug, user_id, fields, submitters, documents, created_at, updated_at FROM templates WHERE user_id = $1 ORDER BY created_at DESC"
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
                fields: row.try_get("fields")?,
                submitters: row.try_get("submitters")?,
                documents: row.try_get("documents")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            });
        }
        Ok(templates)
    }

    pub async fn update_template(pool: &PgPool, id: i64, user_id: i64, name: Option<&str>, fields: Option<&serde_json::Value>, submitters: Option<&serde_json::Value>) -> Result<Option<DbTemplate>, sqlx::Error> {
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            UPDATE templates
            SET name = COALESCE($3, name),
                fields = COALESCE($4, fields),
                submitters = COALESCE($5, submitters),
                updated_at = $6
            WHERE id = $1 AND user_id = $2
            RETURNING id, name, slug, user_id, fields, submitters, documents, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(user_id)
        .bind(name)
        .bind(fields)
        .bind(submitters)
        .bind(now)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => Ok(Some(DbTemplate {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                slug: row.try_get("slug")?,
                user_id: row.try_get("user_id")?,
                fields: row.try_get("fields")?,
                submitters: row.try_get("submitters")?,
                documents: row.try_get("documents")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            })),
            None => Ok(None),
        }
    }

    pub async fn delete_template(pool: &PgPool, id: i64, user_id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM templates WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn clone_template(pool: &PgPool, original_id: i64, user_id: i64, new_name: &str, new_slug: &str) -> Result<Option<DbTemplate>, sqlx::Error> {
        // First get the original template
        if let Some(original) = Self::get_template_by_id(pool, original_id).await? {
            // Check if the template belongs to the user
            if original.user_id != user_id {
                return Ok(None);
            }

            let now = Utc::now();
            let create_data = CreateTemplate {
                name: new_name.to_string(),
                slug: new_slug.to_string(),
                user_id: user_id,
                fields: original.fields,
                submitters: original.submitters,
                documents: original.documents,
            };

            Self::create_template(pool, create_data).await.map(Some)
        } else {
            Ok(None)
        }
    }
}

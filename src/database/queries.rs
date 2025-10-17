use sqlx::{PgPool, Row};
use chrono::{Utc, DateTime};

use super::models::{DbUser, CreateUser, DbTemplate, CreateTemplate, DbTemplateField, CreateTemplateField, CreateSubmitter, DbSubmitter, DbPaymentRecord, CreatePaymentRecord, DbSignatureData, DbSubscriptionPlan};
use crate::models::role::Role;

// Structured query implementations for better organization
pub struct UserQueries;
pub struct TemplateQueries;
pub struct TemplateFieldQueries;
pub struct SubmitterQueries;

impl UserQueries {
    pub async fn get_user_by_id(pool: &PgPool, id: i64) -> Result<Option<DbUser>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, name, email, password_hash, role, subscription_status, subscription_expires_at, free_usage_count, created_at, updated_at FROM users WHERE id = $1"
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
                subscription_status: row.try_get("subscription_status")?,
                subscription_expires_at: row.try_get("subscription_expires_at")?,
                free_usage_count: row.try_get("free_usage_count")?,
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
            INSERT INTO users (name, email, password_hash, role, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, email, password_hash, role, subscription_status, 
                     subscription_expires_at, free_usage_count, created_at, updated_at
            "#
        )
        .bind(&user_data.name)
        .bind(&user_data.email)
        .bind(&user_data.password_hash)
        .bind(&user_data.role)
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
            subscription_status: row.try_get("subscription_status")?,
            subscription_expires_at: row.try_get("subscription_expires_at")?,
            free_usage_count: row.try_get("free_usage_count")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<DbUser>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, name, email, password_hash, role, subscription_status, subscription_expires_at, free_usage_count, created_at, updated_at FROM users WHERE email = $1"
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
                subscription_status: row.try_get("subscription_status")?,
                subscription_expires_at: row.try_get("subscription_expires_at")?,
                free_usage_count: row.try_get("free_usage_count")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            })),
            None => Ok(None),
        }
    }

}

impl TemplateQueries {
    pub async fn create_template(pool: &PgPool, template_data: CreateTemplate) -> Result<DbTemplate, sqlx::Error> {
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO templates (name, slug, user_id, documents, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, slug, user_id, documents, created_at, updated_at
            "#
        )
        .bind(&template_data.name)
        .bind(&template_data.slug)
        .bind(&template_data.user_id)
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
            // fields: None, // Removed - now stored in template_fields table
            documents: row.get(4),
            created_at: row.get(5),
            updated_at: row.get(6),
        })
    }

    pub async fn get_template_by_id(pool: &PgPool, id: i64) -> Result<Option<DbTemplate>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, name, slug, user_id, documents, created_at, updated_at FROM templates WHERE id = $1"
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
            "SELECT id, name, slug, user_id, documents, created_at, updated_at FROM templates WHERE slug = $1"
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
            "SELECT id, name, slug, user_id, documents, created_at, updated_at FROM templates WHERE user_id = $1 ORDER BY created_at DESC "
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
                // fields: None, // Removed - now stored in template_fields table
                documents: row.try_get("documents")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            });
        }
        Ok(templates)
    }

    pub async fn update_template(pool: &PgPool, id: i64, user_id: i64, name: Option<&str>) -> Result<Option<DbTemplate>, sqlx::Error> {
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            UPDATE templates
            SET name = COALESCE($3, name), updated_at = $4
            WHERE id = $1 AND user_id = $2
            RETURNING id, name, slug, user_id, documents, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(user_id)
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
                // fields: None, // Removed - now stored in template_fields table
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
                // fields: None, // Removed - will be cloned separately via TemplateFieldQueries
                documents: original.documents,
            };

            Self::create_template(pool, create_data).await.map(Some)
        } else {
            Ok(None)
        }
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
            "INSERT INTO submitters (template_id, user_id, name, email, status, signed_at, token, bulk_signatures, ip_address, user_agent, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
             RETURNING id, template_id, user_id, name, email, status, signed_at, token, bulk_signatures, ip_address, user_agent, created_at, updated_at "
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
            created_at: row.get(11),
            updated_at: row.get(12),
        })
    }

    pub async fn get_submitters_by_template(pool: &PgPool, template_id: i64) -> Result<Vec<DbSubmitter>, sqlx::Error> {
        eprintln!("Getting submitters for template_id: {}", template_id);
        let rows = sqlx::query(
            "SELECT id, template_id, user_id, name, email, status, signed_at, token, bulk_signatures, ip_address, user_agent, created_at, updated_at 
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
                created_at: row.get(11),
                updated_at: row.get(12),
            });
        }
        Ok(submitters)
    }

    pub async fn get_submitters_by_user(pool: &PgPool, user_id: i64) -> Result<Vec<DbSubmitter>, sqlx::Error> {
        eprintln!("Getting submitters for user_id: {}", user_id);
        let rows = sqlx::query(
            "SELECT id, template_id, user_id, name, email, status, signed_at, token, bulk_signatures, ip_address, user_agent, created_at, updated_at 
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
                created_at: row.get(11),
                updated_at: row.get(12),
            });
        }
        Ok(submitters)
    }

    pub async fn get_submitter_by_token(pool: &PgPool, token: &str) -> Result<Option<DbSubmitter>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, template_id, user_id, name, email, status, signed_at, token, bulk_signatures, ip_address, user_agent, created_at, updated_at 
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
                created_at: row.get(11),
                updated_at: row.get(12),
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
             RETURNING id, template_id, user_id, name, email, status, signed_at, token, bulk_signatures, ip_address, user_agent, created_at, updated_at "
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
                created_at: row.get(11),
                updated_at: row.get(12),
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
             RETURNING id, template_id, user_id, name, email, status, signed_at, token, bulk_signatures, ip_address, user_agent, created_at, updated_at "
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
                created_at: row.get(11),
                updated_at: row.get(12),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_submitter_by_id(pool: &PgPool, id: i64) -> Result<Option<DbSubmitter>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, template_id, user_id, name, email, status, signed_at, token, bulk_signatures, ip_address, user_agent, created_at, updated_at 
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
                created_at: row.get(11),
                updated_at: row.get(12),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_submitter(pool: &PgPool, id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM submitters WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
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
            SELECT id, template_id, user_id, name, email, status, signed_at, token, bulk_signatures, ip_address, user_agent, created_at, updated_at
            FROM submitters
            WHERE id = $1 AND bulk_signatures IS NOT NULL
            "#
        )
        .bind(submitter_id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => Ok(Some(DbSubmitter {
                id: row.try_get("id")?,
                template_id: row.try_get("template_id")?,
                user_id: row.try_get("user_id")?,
                name: row.try_get("name")?,
                email: row.try_get("email")?,
                status: row.try_get("status")?,
                signed_at: row.try_get("signed_at")?,
                token: row.try_get("token")?,
                bulk_signatures: row.try_get("bulk_signatures")?,
                ip_address: row.try_get("ip_address")?,
                user_agent: row.try_get("user_agent")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
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

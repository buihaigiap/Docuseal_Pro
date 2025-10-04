use sqlx::{PgPool, Row};
use chrono::{Utc, DateTime};

use super::models::{DbUser, CreateUser, DbTemplate, CreateTemplate, DbSubmission, CreateSubmission, DbSubmitter, CreateSubmitter, DbSignaturePosition, DbSignatureData, DbTemplateField, CreateTemplateField/*, DbBulkSignature*/};
use crate::models::signature::{CreateSignaturePosition, CreateSignatureData};

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
            submitters: None, // No longer stored in templates
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
                submitters: None, // No longer stored in templates
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
                submitters: None, // No longer stored in templates
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
                submitters: None, // No longer stored in templates
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
            SET name = COALESCE($3, name),
                updated_at = $4
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
                submitters: None, // No longer stored in templates
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

pub struct TemplateFieldQueries;

impl TemplateFieldQueries {
    pub async fn create_template_field(pool: &PgPool, field_data: CreateTemplateField) -> Result<DbTemplateField, sqlx::Error> {
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO template_fields (
                template_id, name, field_type, required, display_order,
                position, options, metadata, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, template_id, name, field_type, required, display_order,
                     position, options, metadata, created_at, updated_at
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
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    pub async fn get_template_fields(pool: &PgPool, template_id: i64) -> Result<Vec<DbTemplateField>, sqlx::Error> {
        sqlx::query_as::<_, DbTemplateField>(
            "SELECT * FROM template_fields WHERE template_id = $1 ORDER BY display_order"
        )
        .bind(template_id)
        .fetch_all(pool)
        .await
    }

    pub async fn get_template_field_by_id(pool: &PgPool, field_id: i64) -> Result<Option<DbTemplateField>, sqlx::Error> {
        sqlx::query_as::<_, DbTemplateField>(
            "SELECT * FROM template_fields WHERE id = $1"
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
                position = $6, options = $7, metadata = $8, updated_at = $9
            WHERE id = $1
            RETURNING id, template_id, name, field_type, required, display_order,
                     position, options, metadata, created_at, updated_at
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
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            })),
            None => Ok(None),
        }
    }

    pub async fn delete_template_field(pool: &PgPool, field_id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM template_fields WHERE id = $1")
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
                position, options, metadata, created_at, updated_at
            )
            SELECT
                $2 as template_id, name, field_type, required, display_order,
                position, options, metadata, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
            FROM template_fields
            WHERE template_id = $1
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
             WHERE template_id = $1 AND position IS NOT NULL 
             ORDER BY display_order"
        )
        .bind(template_id)
        .fetch_all(pool)
        .await
    }
}

pub struct SubmissionQueries;

impl SubmissionQueries {
    pub async fn create_submission(pool: &PgPool, submission_data: CreateSubmission) -> Result<DbSubmission, sqlx::Error> {
        let now = Utc::now();
        let row = sqlx::query(
            "INSERT INTO submissions (template_id, user_id, status, documents, created_at, updated_at, expires_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7) 
             RETURNING id, template_id, user_id, status, documents, created_at, updated_at, expires_at "
        )
        .bind(submission_data.template_id)
        .bind(submission_data.user_id)
        .bind(submission_data.status)
        .bind(submission_data.documents)
        .bind(now)
        .bind(now)
        .bind(submission_data.expires_at)
        .fetch_one(pool)
        .await?;

        Ok(DbSubmission {
            id: row.get(0),
            template_id: row.get(1),
            user_id: row.get(2),
            status: row.get(3),
            documents: row.get(4),
            created_at: row.get(5),
            updated_at: row.get(6),
            expires_at: row.get(7),
        })
    }

    pub async fn get_submissions_by_user(pool: &PgPool, user_id: i64) -> Result<Vec<DbSubmission>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, template_id, user_id, status, documents, created_at, updated_at, expires_at 
             FROM submissions WHERE user_id = $1 ORDER BY created_at DESC "
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let mut submissions = Vec::new();
        for row in rows {
            submissions.push(DbSubmission {
                id: row.get(0),
                template_id: row.get(1),
                user_id: row.get(2),
                status: row.get(3),
                documents: row.get(4),
                created_at: row.get(5),
                updated_at: row.get(6),
                expires_at: row.get(7),
            });
        }
        Ok(submissions)
    }

    pub async fn get_submissions_by_template(pool: &PgPool, template_id: i64) -> Result<Vec<DbSubmission>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, template_id, user_id, status, documents, created_at, updated_at, expires_at 
             FROM submissions WHERE template_id = $1 ORDER BY created_at DESC "
        )
        .bind(template_id)
        .fetch_all(pool)
        .await?;

        let mut submissions = Vec::new();
        for row in rows {
            submissions.push(DbSubmission {
                id: row.get(0),
                template_id: row.get(1),
                user_id: row.get(2),
                status: row.get(3),
                documents: row.get(4),
                created_at: row.get(5),
                updated_at: row.get(6),
                expires_at: row.get(7),
            });
        }
        Ok(submissions)
    }

    pub async fn get_submission(pool: &PgPool, id: i64, user_id: i64) -> Result<Option<DbSubmission>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, template_id, user_id, status, documents, created_at, updated_at, expires_at 
             FROM submissions WHERE id = $1 AND user_id = $2"
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(DbSubmission {
                id: row.get(0),
                template_id: row.get(1),
                user_id: row.get(2),
                status: row.get(3),
                documents: row.get(4),
                created_at: row.get(5),
                updated_at: row.get(6),
                expires_at: row.get(7),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_submission_by_id(pool: &PgPool, id: i64) -> Result<Option<DbSubmission>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, template_id, user_id, status, documents, created_at, updated_at, expires_at 
             FROM submissions WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(DbSubmission {
                id: row.get(0),
                template_id: row.get(1),
                user_id: row.get(2),
                status: row.get(3),
                documents: row.get(4),
                created_at: row.get(5),
                updated_at: row.get(6),
                expires_at: row.get(7),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_submission(pool: &PgPool, id: i64, user_id: i64, status: Option<&str>) -> Result<Option<DbSubmission>, sqlx::Error> {
        let now = Utc::now();
        let row = sqlx::query(
            "UPDATE submissions SET status = COALESCE($1, status), updated_at = $2 
             WHERE id = $3 AND user_id = $4 
             RETURNING id, template_id, user_id, status, documents, created_at, updated_at, expires_at "
        )
        .bind(status)
        .bind(now)
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(DbSubmission {
                id: row.get(0),
                template_id: row.get(1),
                user_id: row.get(2),
                status: row.get(3),
                documents: row.get(4),
                created_at: row.get(5),
                updated_at: row.get(6),
                expires_at: row.get(7),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_submission(pool: &PgPool, id: i64, user_id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM submissions WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

pub struct SubmitterQueries;

impl SubmitterQueries {
    pub async fn create_submitter(pool: &PgPool, submitter_data: CreateSubmitter) -> Result<DbSubmitter, sqlx::Error> {
        let now = Utc::now();
        eprintln!("Creating submitter: submission_id={}, name={}, email={}, token={}", 
            submitter_data.submission_id, submitter_data.name, submitter_data.email, submitter_data.token);
        let row = sqlx::query(
            "INSERT INTO submitters (submission_id, name, email, status, signed_at, token, fields_data, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) 
             RETURNING id, submission_id, name, email, status, signed_at, token, fields_data, created_at, updated_at "
        )
        .bind(submitter_data.submission_id)
        .bind(submitter_data.name)
        .bind(submitter_data.email)
        .bind(submitter_data.status)
        .bind(None as Option<DateTime<Utc>>)
        .bind(submitter_data.token)
        .bind(submitter_data.fields_data)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await?;

        eprintln!("Submitter created successfully: id={}", row.get::<i64, _>(0));
        Ok(DbSubmitter {
            id: row.get(0),
            submission_id: row.get(1),
            name: row.get(2),
            email: row.get(3),
            status: row.get(4),
            signed_at: row.get(5),
            token: row.get(6),
            fields_data: row.get(7),
            created_at: row.get(8),
            updated_at: row.get(9),
        })
    }

    pub async fn get_submitters_by_submission(pool: &PgPool, submission_id: i64) -> Result<Vec<DbSubmitter>, sqlx::Error> {
        eprintln!("Getting submitters for submission_id: {}", submission_id);
        let rows = sqlx::query(
            "SELECT id, submission_id, name, email, status, signed_at, token, fields_data, created_at, updated_at 
             FROM submitters WHERE submission_id = $1 ORDER BY created_at "
        )
        .bind(submission_id)
        .fetch_all(pool)
        .await?;

        eprintln!("Found {} submitters", rows.len());
        let mut submitters = Vec::new();
        for row in rows {
            submitters.push(DbSubmitter {
                id: row.get(0),
                submission_id: row.get(1),
                name: row.get(2),
                email: row.get(3),
                status: row.get(4),
                signed_at: row.get(5),
                token: row.get(6),
                fields_data: row.get(7),
                created_at: row.get(8),
                updated_at: row.get(9),
            });
        }
        Ok(submitters)
    }

    pub async fn get_submitter_by_token(pool: &PgPool, token: &str) -> Result<Option<DbSubmitter>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, submission_id, name, email, status, signed_at, token, fields_data, created_at, updated_at 
             FROM submitters WHERE token = $1"
        )
        .bind(token)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(DbSubmitter {
                id: row.get(0),
                submission_id: row.get(1),
                name: row.get(2),
                email: row.get(3),
                status: row.get(4),
                signed_at: row.get(5),
                token: row.get(6),
                fields_data: row.get(7),
                created_at: row.get(8),
                updated_at: row.get(9),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_submitter(pool: &PgPool, id: i64, status: Option<&str>, fields_data: Option<&serde_json::Value>) -> Result<Option<DbSubmitter>, sqlx::Error> {
        let now = Utc::now();
        let signed_at = if status == Some("signed") { Some(now) } else { None };
        
        let row = sqlx::query(
            "UPDATE submitters SET status = COALESCE($1, status), fields_data = COALESCE($2, fields_data), signed_at = COALESCE($3, signed_at), updated_at = $4 
             WHERE id = $5 
             RETURNING id, submission_id, name, email, status, signed_at, token, fields_data, created_at, updated_at "
        )
        .bind(status)
        .bind(fields_data)
        .bind(signed_at)
        .bind(now)
        .bind(id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(DbSubmitter {
                id: row.get(0),
                submission_id: row.get(1),
                name: row.get(2),
                email: row.get(3),
                status: row.get(4),
                signed_at: row.get(5),
                token: row.get(6),
                fields_data: row.get(7),
                created_at: row.get(8),
                updated_at: row.get(9),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_submitter_by_id(pool: &PgPool, id: i64) -> Result<Option<DbSubmitter>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, submission_id, name, email, status, signed_at, token, fields_data, created_at, updated_at 
             FROM submitters WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(DbSubmitter {
                id: row.get(0),
                submission_id: row.get(1),
                name: row.get(2),
                email: row.get(3),
                status: row.get(4),
                signed_at: row.get(5),
                token: row.get(6),
                fields_data: row.get(7),
                created_at: row.get(8),
                updated_at: row.get(9),
            }))
        } else {
            Ok(None)
        }
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

    pub async fn create_bulk_signature_position(
        pool: &PgPool,
        submitter_id: i64,
        bulk_signatures: serde_json::Value,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<DbSignaturePosition, sqlx::Error> {
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO signature_positions (submitter_id, bulk_signatures, signed_at, ip_address, user_agent, version, is_active, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, submitter_id, bulk_signatures, signed_at, ip_address, user_agent, version, is_active, created_at, updated_at, signature_image
            "#
        )
        .bind(submitter_id)
        .bind(&bulk_signatures)
        .bind(now)
        .bind(&ip_address)
        .bind(&user_agent)
        .bind(1) // version
        .bind(Some(true)) // is_active
        .bind(now)
        .fetch_one(pool)
        .await?;

        Ok(DbSignaturePosition {
            id: row.try_get("id")?,
            submitter_id: row.try_get("submitter_id")?,
            bulk_signatures: row.try_get("bulk_signatures")?,
            signed_at: row.try_get("signed_at")?,
            ip_address: row.try_get("ip_address")?,
            user_agent: row.try_get("user_agent")?,
            version: row.try_get("version")?,
            is_active: row.try_get("is_active")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            signature_image: row.try_get("signature_image")?,
        })
    }

    pub async fn get_bulk_signature_positions_by_submitter(
        pool: &PgPool,
        submitter_id: i64,
    ) -> Result<Vec<DbSignaturePosition>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, submitter_id, bulk_signatures, signed_at, ip_address, user_agent, version, is_active, created_at, updated_at, signature_image
            FROM signature_positions
            WHERE submitter_id = $1 AND bulk_signatures IS NOT NULL AND is_active = true
            ORDER BY created_at DESC
            "#
        )
        .bind(submitter_id)
        .fetch_all(pool)
        .await?;

        let mut positions = Vec::new();
        for row in rows {
            positions.push(DbSignaturePosition {
                id: row.try_get("id")?,
                submitter_id: row.try_get("submitter_id")?,
                bulk_signatures: row.try_get("bulk_signatures")?,
                signed_at: row.try_get("signed_at")?,
                ip_address: row.try_get("ip_address")?,
                user_agent: row.try_get("user_agent")?,
                version: row.try_get("version")?,
                is_active: row.try_get("is_active")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
                signature_image: row.try_get("signature_image")?,
            });
        }

        Ok(positions)
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

// DEPRECATED: Bulk signatures now stored in signature_positions table with bulk_signatures column
// pub struct BulkSignatureQueries;

// impl BulkSignatureQueries {
//     pub async fn create_bulk_signature(
//         pool: &PgPool,
//         submitter_id: i64,
//         signatures: &serde_json::Value,
//         ip_address: Option<&str>,
//         user_agent: Option<&str>,
//     ) -> Result<DbBulkSignature, sqlx::Error> {
//         // Ensure table exists
//         let _ = sqlx::query(
//             r#"
//             CREATE TABLE IF NOT EXISTS bulk_signatures (
//                 id BIGSERIAL PRIMARY KEY,
//                 submitter_id BIGINT NOT NULL REFERENCES submitters(id) ON DELETE CASCADE,
//                 signatures JSONB NOT NULL, -- Array of {field_id, field_name, signature_value} objects
//                 signed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
//                 ip_address TEXT,
//                 user_agent TEXT,
//                 created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
//                 updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
//             )
//             "#
//         )
//         .execute(pool)
//         .await;

//         // Create indexes if they don't exist
//         let _ = sqlx::query(
//             r#"
//             CREATE INDEX IF NOT EXISTS idx_bulk_signatures_submitter_id ON bulk_signatures(submitter_id);
//             CREATE INDEX IF NOT EXISTS idx_bulk_signatures_signed_at ON bulk_signatures(signed_at);
//             "#
//         )
//         .execute(pool)
//         .await;

//         let row = sqlx::query(
//             r#"
//             INSERT INTO bulk_signatures (submitter_id, signatures, ip_address, user_agent, signed_at)
//             VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP)
//             RETURNING id, submitter_id, signatures, signed_at, ip_address, user_agent, created_at, updated_at
//             "#
//         )
//         .bind(submitter_id)
//         .bind(signatures)
//         .bind(ip_address)
//         .bind(user_agent)
//         .fetch_one(pool)
//         .await?;

//         Ok(DbBulkSignature {
//             id: row.try_get("id")?,
//             submitter_id: row.try_get("submitter_id")?,
//             signatures: row.try_get("signatures")?,
//             signed_at: row.try_get("signed_at")?,
//             ip_address: row.try_get("ip_address")?,
//             user_agent: row.try_get("user_agent")?,
//             created_at: row.try_get("created_at")?,
//             updated_at: row.try_get("updated_at")?,
//         })
//     }

//     pub async fn get_bulk_signatures_by_submitter(
//         pool: &PgPool,
//         submitter_id: i64,
//     ) -> Result<Vec<DbBulkSignature>, sqlx::Error> {
//         let rows = sqlx::query(
//             r#"
//             SELECT id, submitter_id, signatures, signed_at, ip_address, user_agent, created_at, updated_at
//             FROM bulk_signatures
//             WHERE submitter_id = $1
//             ORDER BY created_at DESC
//             "#
//         )
//         .bind(submitter_id)
//         .fetch_all(pool)
//         .await?;

//         let mut signatures = Vec::new();
//         for row in rows {
//             signatures.push(DbBulkSignature {
//                 id: row.try_get("id")?,
//                 submitter_id: row.try_get("submitter_id")?,
//                 signatures: row.try_get("signatures")?,
//                 signed_at: row.try_get("signed_at")?,
//                 ip_address: row.try_get("ip_address")?,
//                 user_agent: row.try_get("user_agent")?,
//                 created_at: row.try_get("created_at")?,
//                 updated_at: row.try_get("updated_at")?,
//             });
//         }
//         Ok(signatures)
//     }

//     pub async fn get_latest_bulk_signature_by_submitter(
//         pool: &PgPool,
//         submitter_id: i64,
//     ) -> Result<Option<DbBulkSignature>, sqlx::Error> {
//         let row = sqlx::query(
//             r#"
//             SELECT id, submitter_id, signatures, signed_at, ip_address, user_agent, created_at, updated_at
//             FROM bulk_signatures
//             WHERE submitter_id = $1
//             ORDER BY created_at DESC
//             LIMIT 1
//             "#
//         )
//         .bind(submitter_id)
//         .fetch_optional(pool)
//         .await?;

//         if let Some(row) = row {
//             Ok(Some(DbBulkSignature {
//                 id: row.try_get("id")?,
//                 submitter_id: row.try_get("submitter_id")?,
//                 signatures: row.try_get("signatures")?,
//                 signed_at: row.try_get("signed_at")?,
//                 ip_address: row.try_get("ip_address")?,
//                 user_agent: row.try_get("user_agent")?,
//                 created_at: row.try_get("created_at")?,
//                 updated_at: row.try_get("updated_at")?,
//             }))
//         } else {
//             Ok(None)
//         }
//     }
// }
//         )
//         .bind(submitter_id)
//         .fetch_all(pool)
//         .await?;

//         let mut signatures = Vec::new();
//         for row in rows {
//             signatures.push(DbBulkSignature {
//                 id: row.try_get("id")?,
//                 submitter_id: row.try_get("submitter_id")?,
//                 signatures: row.try_get("signatures")?,
//                 signed_at: row.try_get("signed_at")?,
//                 ip_address: row.try_get("ip_address")?,
//                 user_agent: row.try_get("user_agent")?,
//                 created_at: row.try_get("created_at")?,
//                 updated_at: row.try_get("updated_at")?,
//             });
//         }
//         Ok(signatures)
//     }

//     pub async fn get_latest_bulk_signature_by_submitter(
//         pool: &PgPool,
//         submitter_id: i64,
//     ) -> Result<Option<DbBulkSignature>, sqlx::Error> {
//         let row = sqlx::query(
//             r#"
//             SELECT id, submitter_id, signatures, signed_at, ip_address, user_agent, created_at, updated_at
//             FROM bulk_signatures
//             WHERE submitter_id = $1
//             ORDER BY created_at DESC
//             LIMIT 1
//             "#
//         )
//         .bind(submitter_id)
//         .fetch_optional(pool)
//         .await?;

//         if let Some(row) = row {
//             Ok(Some(DbBulkSignature {
//                 id: row.try_get("id")?,
//                 submitter_id: row.try_get("submitter_id")?,
//                 signatures: row.try_get("signatures")?,
//                 signed_at: row.try_get("signed_at")?,
//                 ip_address: row.try_get("ip_address")?,
//                 user_agent: row.try_get("user_agent")?,
//                 created_at: row.try_get("created_at")?,
//                 updated_at: row.try_get("updated_at")?,
//             }))
//         } else {
//             Ok(None)
//         }
//     }
// }

    }

    // Calculate text height dynamically (matching SignatureRenderer.tsx)
    let text_height = calculate_signature_text_height(
        global_settings,
        Some(submitter.id),
        &signer_email,
        reason
    );

    // Position the signature info at the BOTTOM of the field
    // Matching SignatureRenderer.tsx: text starts from bottom and goes up
    let info_x = x_pos + 5.0; // Match frontend padding of 5px
    let font_size = 8.0; // Match frontend font size
    let line_height = 10.0; // Match frontend line height
    
    // Text area is at the bottom: from pdf_y to (pdf_y + text_height)
    // Calculate actual text height needed
    let actual_text_height = (signature_info_parts.len() as f64 - 1.0) * line_height + font_size + 3.0;
    
    // Start rendering from the bottom of text area
    // First line should be at pdf_y + 3 (bottom padding), last line at the top
    let text_start_y = pdf_y + 3.0; // Bottom padding: 3px from the bottom
    
    // Create text content stream for signature info with multiple lines
    let mut text_operations = vec![
        Operation::new("BT", vec![]), // Begin text
        Operation::new("Tf", vec![
            Object::Name(b"Helvetica".to_vec()),
            Object::Real(font_size as f32),
        ]), // Set font
        Operation::new("rg", vec![
            Object::Real(0.0),
            Object::Real(0.0),
            Object::Real(0.0),
        ]), // Set text color to black
    ];
    
    // Render each line from bottom to top (matching SignatureRenderer.tsx)
    // SignatureRenderer draws: for (let i = textToShow.length - 1; i >= 0; i--)
    let num_lines = signature_info_parts.len();
    for (idx, line) in signature_info_parts.iter().enumerate() {
        // Calculate Y position: start from bottom and go up
        // Line 0 (first in array) at bottom, line N-1 (last) at top
        let line_y = text_start_y + ((num_lines - 1 - idx) as f64 * line_height);
        
        // Use Tm (text matrix) to set absolute position for each line
        text_operations.push(Operation::new("Tm", vec![
            Object::Real(1.0), // a: horizontal scaling
            Object::Real(0.0), // b: horizontal skewing
            Object::Real(0.0), // c: vertical skewing
            Object::Real(1.0), // d: vertical scaling
            Object::Real(info_x as f32), // e: horizontal position
            Object::Real(line_y as f32),  // f: vertical position
        ]));
        
        text_operations.push(Operation::new("Tj", vec![
            Object::string_literal(line.clone()),
        ])); // Show text
    }
    
    text_operations.push(Operation::new("ET", vec![])); // End text
    
    let content = Content { operations: text_operations };
    let content_data = content.encode()?;
    
    // Create a new content stream
    let stream = Stream::new(Dictionary::new(), content_data);
    let stream_id = doc.add_object(stream);
    
    // Get the page object and add stream to it
    if let Ok(page_obj) = doc.get_object_mut(page_id) {
        if let Ok(page_dict) = page_obj.as_dict_mut() {
            // Add to page's content array
            if let Ok(contents_obj) = page_dict.get_mut(b"Contents") {
                match contents_obj {
                    Object::Reference(_ref_id) => {
                        // For simplicity, replace the content reference with our new stream
                        *contents_obj = Object::Reference(stream_id);
                    },
                    Object::Array(ref mut contents_array) => {
                        contents_array.push(Object::Reference(stream_id));
                    },
                    _ => {
                        // Replace with new content stream
                        *contents_obj = Object::Reference(stream_id);
                    }
                }
            } else {
                // Add new Contents array
                page_dict.set(b"Contents", Object::Reference(stream_id));
            }
        }
    }
    
    Ok(())
}

// Helper function to extract filename from URL
fn extract_filename_from_url(url: &str) -> String {
    url.split('/').last().unwrap_or("file").to_string()
}

// Helper function to generate hash ID similar to frontend hashId function
fn hash_id(value: i64) -> String {
    let str_value = value.to_string();
    
    // Create 32-bit hash from value
    let mut hash: i32 = 0;
    for ch in str_value.chars() {
        hash = ((hash << 5).wrapping_sub(hash).wrapping_add(ch as i32)) | 0;
    }
    
    // Generate hex string (8 characters from 32-bit hash)
    let mut hex = String::new();
    for i in 0..8 {
        let h = ((hash >> (i * 4)) & 0xF) as u8;
        hex.push_str(&format!("{:X}", h));
    }
    
    // Repeat to get 32 characters: hex.len() = 8, we need 32, so repeat 4 times
    let hex32 = format!("{}{}{}{}", hex, hex, hex, hex);
    
    // Format as UUID (8-4-4-4-12 = 32 characters)
    format!(
        "{}-{}-{}-{}-{}",
        &hex32[0..8],
        &hex32[8..12],
        &hex32[12..16],
        &hex32[16..20],
        &hex32[20..32]
    )
}

// Calculate text height for signature info (matching SignatureRenderer.tsx logic)
fn calculate_signature_text_height(
    global_settings: &crate::database::models::DbGlobalSettings,
    submitter_id: Option<i64>,
    submitter_email: &str,
    reason: &str,
) -> f64 {
    let mut line_count = 0;
    
    if global_settings.add_signature_id_to_the_documents {
        if submitter_id.is_some() { line_count += 1; }
        if !submitter_email.is_empty() { line_count += 1; }
        line_count += 1; // date
    }
    
    if global_settings.require_signing_reason && !reason.is_empty() {
        line_count += 1;
    }
    
    // Match SignatureRenderer.tsx: (lineCount - 1) * 10 + 8 + 3
    if line_count > 0 {
        ((line_count - 1) as f64 * 10.0) + 8.0 + 3.0
    } else {
        0.0
    }
}

// Render signature ID information below the signature
fn render_signature_id_info(
    doc: &mut lopdf::Document,
    page_id: lopdf::ObjectId,
    submitter: &crate::database::models::DbSubmitter,
    signature_data: &serde_json::Value,
    x_pos: f64,
    pdf_y: f64,
    field_width: f64,
    field_height: f64,
    global_settings: &crate::database::models::DbGlobalSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    use lopdf::{Object, Stream, Dictionary};
    use lopdf::content::{Content, Operation};

    // Generate signature ID using hashId function (matching frontend)
    let signature_id = hash_id(submitter.id + 1);

    // Get reason from signature data
    let reason = signature_data.get("reason")
        .and_then(|r| r.as_str())
        .unwrap_or("");

    // Format the signature information
    let signer_email = submitter.email.clone();
    let signed_at = submitter.signed_at.unwrap_or(chrono::Utc::now());
    
    // Parse timezone from global settings or use default GMT+7
    let timezone_str = global_settings.timezone.as_deref().unwrap_or("Asia/Ho_Chi_Minh");
    
    // Map common timezone names to IANA identifiers (matching SignatureRenderer)
    let timezone_mapped = match timezone_str {
        "Midway Island" => "Pacific/Midway",
        "Hawaii" => "Pacific/Honolulu",
        "Alaska" => "America/Anchorage",
        "Pacific" => "America/Los_Angeles",
        "Mountain" => "America/Denver",
        "Central" => "America/Chicago",
        "Eastern" => "America/New_York",
        "Atlantic" => "America/Halifax",
        "Newfoundland" => "America/St_Johns",
        "London" => "Europe/London",
        "Berlin" => "Europe/Berlin",
        "Paris" => "Europe/Paris",
        "Rome" => "Europe/Rome",
        "Moscow" => "Europe/Moscow",
        "Tokyo" => "Asia/Tokyo",
        "Shanghai" => "Asia/Shanghai",
        "Hong Kong" => "Asia/Hong_Kong",
        "Singapore" => "Asia/Singapore",
        "Sydney" => "Australia/Sydney",
        "UTC" => "UTC",
        _ => timezone_str,
    };
    
    // Parse timezone offset (simplified approach for common timezones)
    let timezone_offset_hours = match timezone_mapped {
        "Asia/Ho_Chi_Minh" => 7,
        "Pacific/Midway" => -11,
        "Pacific/Honolulu" => -10,
        "America/Anchorage" => -9,
        "America/Los_Angeles" => -8,
        "America/Denver" => -7,
        "America/Chicago" => -6,
        "America/New_York" => -5,
        "America/Halifax" => -4,
        "Europe/London" => 0,
        "Europe/Berlin" | "Europe/Paris" | "Europe/Rome" => 1,
        "Europe/Moscow" => 3,
        "Asia/Tokyo" => 9,
        "Asia/Shanghai" | "Asia/Hong_Kong" | "Asia/Singapore" => 8,
        "Australia/Sydney" => 10,
        "UTC" => 0,
        _ => 7, // Default to GMT+7
    };
    
    let timezone_offset = chrono::FixedOffset::east_opt(timezone_offset_hours * 3600).unwrap();
    let signed_at_formatted = signed_at.with_timezone(&timezone_offset);
    
    // Format date according to locale (simplified)
    let locale = global_settings.locale.as_deref().unwrap_or("vi-VN");
    let date_str = if locale.starts_with("vi") {
        // Vietnamese format: DD/MM/YYYY, HH:MM:SS
        signed_at_formatted.format("%d/%m/%Y, %H:%M:%S").to_string()
    } else {
        // English/Default format: MM/DD/YYYY, HH:MM:SS
        signed_at_formatted.format("%m/%d/%Y, %H:%M:%S").to_string()
    };
    
    let mut signature_info_parts = Vec::new();
    
    // Always show reason first if require_signing_reason is enabled and reason exists
    if global_settings.require_signing_reason && !reason.is_empty() {
        signature_info_parts.push(format!("Reason: {}", reason));
    }
    
    // Show ID, email, and date if add_signature_id_to_the_documents is enabled
    if global_settings.add_signature_id_to_the_documents {
        signature_info_parts.push(format!("ID: {}", signature_id));
        signature_info_parts.push(signer_email.clone());
        signature_info_parts.push(date_str);
    }
    
    // If nothing to show, return early
    if signature_info_parts.is_empty() {
        return Ok(());
    }

    // Calculate text height dynamically (matching SignatureRenderer.tsx)
    let text_height = calculate_signature_text_height(
        global_settings,
        Some(submitter.id),
        &signer_email,
        reason
    );

    // Position the signature info at the BOTTOM of the field
    // Matching SignatureRenderer.tsx: text starts from bottom and goes up
    let info_x = x_pos + 5.0; // Match frontend padding of 5px
    let font_size = 8.0; // Match frontend font size
    let line_height = 10.0; // Match frontend line height
    
    // Text area is at the bottom: from pdf_y to (pdf_y + text_height)
    // Calculate actual text height needed
    let actual_text_height = (signature_info_parts.len() as f64 - 1.0) * line_height + font_size + 3.0;
    
    // Start rendering from the bottom of text area
    // First line should be at pdf_y + 3 (bottom padding), last line at the top
    let text_start_y = pdf_y + 3.0; // Bottom padding: 3px from the bottom
    
    // Create text content stream for signature info with multiple lines
    let mut text_operations = vec![
        Operation::new("BT", vec![]), // Begin text
        Operation::new("Tf", vec![
            Object::Name(b"Helvetica".to_vec()),
            Object::Real(font_size as f32),
        ]), // Set font
        Operation::new("rg", vec![
            Object::Real(0.0),
            Object::Real(0.0),
            Object::Real(0.0),
        ]), // Set text color to black
    ];
    
    // Render each line from bottom to top (matching SignatureRenderer.tsx)
    // SignatureRenderer draws: for (let i = textToShow.length - 1; i >= 0; i--)
    let num_lines = signature_info_parts.len();
    for (idx, line) in signature_info_parts.iter().enumerate() {
        // Calculate Y position: start from bottom and go up
        // Line 0 (first in array) at bottom, line N-1 (last) at top
        let line_y = text_start_y + ((num_lines - 1 - idx) as f64 * line_height);
        
        // Use Tm (text matrix) to set absolute position for each line
        text_operations.push(Operation::new("Tm", vec![
            Object::Real(1.0), // a: horizontal scaling
            Object::Real(0.0), // b: horizontal skewing
            Object::Real(0.0), // c: vertical skewing
            Object::Real(1.0), // d: vertical scaling
            Object::Real(info_x as f32), // e: horizontal position
            Object::Real(line_y as f32),  // f: vertical position
        ]));
        
        text_operations.push(Operation::new("Tj", vec![
            Object::string_literal(line.clone()),
        ])); // Show text
    }
    
    text_operations.push(Operation::new("ET", vec![])); // End text
    
    let content = Content { operations: text_operations };
    let content_data = content.encode()?;
    
    // Create a new content stream
    let stream = Stream::new(Dictionary::new(), content_data);
    let stream_id = doc.add_object(stream);
    
    // Get the page object and add stream to it
    if let Ok(page_obj) = doc.get_object_mut(page_id) {
        if let Ok(page_dict) = page_obj.as_dict_mut() {
            // Add to page's content array
            if let Ok(contents_obj) = page_dict.get_mut(b"Contents") {
                match contents_obj {
                    Object::Reference(_ref_id) => {
                        // For simplicity, replace the content reference with our new stream
                        *contents_obj = Object::Reference(stream_id);
                    },
                    Object::Array(ref mut contents_array) => {
                        contents_array.push(Object::Reference(stream_id));
                    },
                    _ => {
                        // Replace with new content stream
                        *contents_obj = Object::Reference(stream_id);
                    }
                }
            } else {
                // Add new Contents array
                page_dict.set(b"Contents", Object::Reference(stream_id));
            }
        }
    }
    
    Ok(())
}

// Helper function to extract filename from URL
fn extract_filename_from_url(url: &str) -> String {
    url.split('/').last().unwrap_or("file").to_string()
}

// Helper function to generate hash ID similar to frontend hashId function
fn hash_id(value: i64) -> String {
    let str_value = value.to_string();
    
    // Create 32-bit hash from value
    let mut hash: i32 = 0;
    for ch in str_value.chars() {
        hash = ((hash << 5).wrapping_sub(hash).wrapping_add(ch as i32)) | 0;
    }
    
    // Generate hex string (8 characters from 32-bit hash)
    let mut hex = String::new();
    for i in 0..8 {
        let h = ((hash >> (i * 4)) & 0xF) as u8;
        hex.push_str(&format!("{:X}", h));
    }
    
    // Repeat to get 32 characters: hex.len() = 8, we need 32, so repeat 4 times
    let hex32 = format!("{}{}{}{}", hex, hex, hex, hex);
    
    // Format as UUID (8-4-4-4-12 = 32 characters)
    format!(
        "{}-{}-{}-{}-{}",
        &hex32[0..8],
        &hex32[8..12],
        &hex32[12..16],
        &hex32[16..20],
        &hex32[20..32]
    )
}

// Calculate text height for signature info (matching SignatureRenderer.tsx logic)
fn calculate_signature_text_height(
    global_settings: &crate::database::models::DbGlobalSettings,
    submitter_id: Option<i64>,
    submitter_email: &str,
    reason: &str,
) -> f64 {
    let mut line_count = 0;
    
    if global_settings.add_signature_id_to_the_documents {
        if submitter_id.is_some() { line_count += 1; }
        if !submitter_email.is_empty() { line_count += 1; }
        line_count += 1; // date
    }
    
    if global_settings.require_signing_reason && !reason.is_empty() {
        line_count += 1;
    }
    
    // Match SignatureRenderer.tsx: (lineCount - 1) * 10 + 8 + 3
    if line_count > 0 {
        ((line_count - 1) as f64 * 10.0) + 8.0 + 3.0
    } else {
        0.0
    }
}

// Render signature ID information below the signature
fn render_signature_id_info(
    doc: &mut lopdf::Document,
    page_id: lopdf::ObjectId,
    submitter: &crate::database::models::DbSubmitter,
    signature_data: &serde_json::Value,
    x_pos: f64,
    pdf_y: f64,
    field_width: f64,
    field_height: f64,
    global_settings: &crate::database::models::DbGlobalSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    use lopdf::{Object, Stream, Dictionary};
    use lopdf::content::{Content, Operation};

    // Generate signature ID using hashId function (matching frontend)
    let signature_id = hash_id(submitter.id + 1);

    // Get reason from signature data
    let reason = signature_data.get("reason")
        .and_then(|r| r.as_str())
        .unwrap_or("");

    // Format the signature information
    let signer_email = submitter.email.clone();
    let signed_at = submitter.signed_at.unwrap_or(chrono::Utc::now());
    
    // Parse timezone from global settings or use default GMT+7
    let timezone_str = global_settings.timezone.as_deref().unwrap_or("Asia/Ho_Chi_Minh");
    
    // Map common timezone names to IANA identifiers (matching SignatureRenderer)
    let timezone_mapped = match timezone_str {
        "Midway Island" => "Pacific/Midway",
        "Hawaii" => "Pacific/Honolulu",
        "Alaska" => "America/Anchorage",
        "Pacific" => "America/Los_Angeles",
        "Mountain" => "America/Denver",
        "Central" => "America/Chicago",
        "Eastern" => "America/New_York",
        "Atlantic" => "America/Halifax",
        "Newfoundland" => "America/St_Johns",
        "London" => "Europe/London",
        "Berlin" => "Europe/Berlin",
        "Paris" => "Europe/Paris",
        "Rome" => "Europe/Rome",
        "Moscow" => "Europe/Moscow",
        "Tokyo" => "Asia/Tokyo",
        "Shanghai" => "Asia/Shanghai",
        "Hong Kong" => "Asia/Hong_Kong",
        "Singapore" => "Asia/Singapore",
        "Sydney" => "Australia/Sydney",
        "UTC" => "UTC",
        _ => timezone_str,
    };
    
    // Parse timezone offset (simplified approach for common timezones)
    let timezone_offset_hours = match timezone_mapped {
        "Asia/Ho_Chi_Minh" => 7,
        "Pacific/Midway" => -11,
        "Pacific/Honolulu" => -10,
        "America/Anchorage" => -9,
        "America/Los_Angeles" => -8,
        "America/Denver" => -7,
        "America/Chicago" => -6,
        "America/New_York" => -5,
        "America/Halifax" => -4,
        "Europe/London" => 0,
        "Europe/Berlin" | "Europe/Paris" | "Europe/Rome" => 1,
        "Europe/Moscow" => 3,
        "Asia/Tokyo" => 9,
        "Asia/Shanghai" | "Asia/Hong_Kong" | "Asia/Singapore" => 8,
        "Australia/Sydney" => 10,
        "UTC" => 0,
        _ => 7, // Default to GMT+7
    };
    
    let timezone_offset = chrono::FixedOffset::east_opt(timezone_offset_hours * 3600).unwrap();
    let signed_at_formatted = signed_at.with_timezone(&timezone_offset);
    
    // Format date according to locale (simplified)
    let locale = global_settings.locale.as_deref().unwrap_or("vi-VN");
    let date_str = if locale.starts_with("vi") {
        // Vietnamese format: DD/MM/YYYY, HH:MM:SS
        signed_at_formatted.format("%d/%m/%Y, %H:%M:%S").to_string()
    } else {
        // English/Default format: MM/DD/YYYY, HH:MM:SS
        signed_at_formatted.format("%m/%d/%Y, %H:%M:%S").to_string()
    };
    
    let mut signature_info_parts = Vec::new();
    
    // Always show reason first if require_signing_reason is enabled and reason exists
    if global_settings.require_signing_reason && !reason.is_empty() {
        signature_info_parts.push(format!("Reason: {}", reason));
    }
    
    // Show ID, email, and date if add_signature_id_to_the_documents is enabled
    if global_settings.add_signature_id_to_the_documents {
        signature_info_parts.push(format!("ID: {}", signature_id));
        signature_info_parts.push(signer_email.clone());
        signature_info_parts.push(date_str);
    }
    
    // If nothing to show, return early
    if signature_info_parts.is_empty() {
        return Ok(());
    }

    // Calculate text height dynamically (matching SignatureRenderer.tsx)
    let text_height = calculate_signature_text_height(
        global_settings,
        Some(submitter.id),
        &signer_email,
        reason
    );

    // Position the signature info at the BOTTOM of the field
    // Matching SignatureRenderer.tsx: text starts from bottom and goes up
    let info_x = x_pos + 5.0; // Match frontend padding of 5px
    let font_size = 8.0; // Match frontend font size
    let line_height = 10.0; // Match frontend line height
    
    // Text area is at the bottom: from pdf_y to (pdf_y + text_height)
    // Calculate actual text height needed
    let actual_text_height = (signature_info_parts.len() as f64 - 1.0) * line_height + font_size + 3.0;
    
    // Start rendering from the bottom of text area
    // First line should be at pdf_y + 3 (bottom padding), last line at the top
    let text_start_y = pdf_y + 3.0; // Bottom padding: 3px from the bottom
    
    // Create text content stream for signature info with multiple lines
    let mut text_operations = vec![
        Operation::new("BT", vec![]), // Begin text
        Operation::new("Tf", vec![
            Object::Name(b"Helvetica".to_vec()),
            Object::Real(font_size as f32),
        ]), // Set font
        Operation::new("rg", vec![
            Object::Real(0.0),
            Object::Real(0.0),
            Object::Real(0.0),
        ]), // Set text color to black
    ];
    
    // Render each line from bottom to top (matching SignatureRenderer.tsx)
    // SignatureRenderer draws: for (let i = textToShow.length - 1; i >= 0; i--)
    let num_lines = signature_info_parts.len();
    for (idx, line) in signature_info_parts.iter().enumerate() {
        // Calculate Y position: start from bottom and go up
        // Line 0 (first in array) at bottom, line N-1 (last) at top
        let line_y = text_start_y + ((num_lines - 1 - idx) as f64 * line_height);
        
        // Use Tm (text matrix) to set absolute position for each line
        text_operations.push(Operation::new("Tm", vec![
            Object::Real(1.0), // a: horizontal scaling
            Object::Real(0.0), // b: horizontal skewing
            Object::Real(0.0), // c: vertical skewing
            Object::Real(1.0), // d: vertical scaling
            Object::Real(info_x as f32), // e: horizontal position
            Object::Real(line_y as f32),  // f: vertical position
        ]));
        
        text_operations.push(Operation::new("Tj", vec![
            Object::string_literal(line.clone()),
        ])); // Show text
    }
    
    text_operations.push(Operation::new("ET", vec![])); // End text
    
    let content = Content { operations: text_operations };
    let content_data = content.encode()?;
    
    // Create a new content stream
    let stream = Stream::new(Dictionary::new(), content_data);
    let stream_id = doc.add_object(stream);
    
    // Get the page object and add stream to it
    if let Ok(page_obj) = doc.get_object_mut(page_id) {
        if let Ok(page_dict) = page_obj.as_dict_mut() {
            // Add to page's content array
            if let Ok(contents_obj) = page_dict.get_mut(b"Contents") {
                match contents_obj {
                    Object::Reference(_ref_id) => {
                        // For simplicity, replace the content reference with our new stream
                        *contents_obj = Object::Reference(stream_id);
                    },
                    Object::Array(ref mut contents_array) => {
                        contents_array.push(Object::Reference(stream_id));
                    },
                    _ => {
                        // Replace with new content stream
                        *contents_obj = Object::Reference(stream_id);
                    }
                }
            } else {
                // Add new Contents array
                page_dict.set(b"Contents", Object::Reference(stream_id));
            }
        }
    }
    
    Ok(())
}

// Helper function to extract filename from URL
fn extract_filename_from_url(url: &str) -> String {
    url.split('/').last().unwrap_or("file").to_string()
}

// Helper function to generate hash ID similar to frontend hashId function
fn hash_id(value: i64) -> String {
    let str_value = value.to_string();
    
    // Create 32-bit hash from value
    let mut hash: i32 = 0;
    for ch in str_value.chars() {
        hash = ((hash << 5).wrapping_sub(hash).wrapping_add(ch as i32)) | 0;
    }
    
    // Generate hex string (8 characters from 32-bit hash)
    let mut hex = String::new();
    for i in 0..8 {
        let h = ((hash >> (i * 4)) & 0xF) as u8;
        hex.push_str(&format!("{:X}", h));
    }
    
    // Repeat to get 32 characters: hex.len() = 8, we need 32, so repeat 4 times
    let hex32 = format!("{}{}{}{}", hex, hex, hex, hex);
    
    // Format as UUID (8-4-4-4-12 = 32 characters)
    format!(
        "{}-{}-{}-{}-{}",
        &hex32[0..8],
        &hex32[8..12],
        &hex32[12..16],
        &hex32[16..20],
        &hex32[20..32]
    )
}

// Calculate text height for signature info (matching SignatureRenderer.tsx logic)
fn calculate_signature_text_height(
    global_settings: &crate::database::models::DbGlobalSettings,
    submitter_id: Option<i64>,
    submitter_email: &str,
    reason: &str,
) -> f64 {
    let mut line_count = 0;
    
    if global_settings.add_signature_id_to_the_documents {
        if submitter_id.is_some() { line_count += 1; }
        if !submitter_email.is_empty() { line_count += 1; }
        line_count += 1; // date
    }
    
    if global_settings.require_signing_reason && !reason.is_empty() {
        line_count += 1;
    }
    
    // Match SignatureRenderer.tsx: (lineCount - 1) * 10 + 8 + 3
    if line_count > 0 {
        ((line_count - 1) as f64 * 10.0) + 8.0 + 3.0
    } else {
        0.0
    }
}

// Render signature ID information below the signature
fn render_signature_id_info(
    doc: &mut lopdf::Document,
    page_id: lopdf::ObjectId,
    submitter: &crate::database::models::DbSubmitter,
    signature_data: &serde_json::Value,
    x_pos: f64,
    pdf_y: f64,
    field_width: f64,
    field_height: f64,
    global_settings: &crate::database::models::DbGlobalSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    use lopdf::{Object, Stream, Dictionary};
    use lopdf::content::{Content, Operation};

    // Generate signature ID using hashId function (matching frontend)
    let signature_id = hash_id(submitter.id + 1);

    // Get reason from signature data
    let reason = signature_data.get("reason")
        .and_then(|r| r.as_str())
        .unwrap_or("");

    // Format the signature information
    let signer_email = submitter.email.clone();
    let signed_at = submitter.signed_at.unwrap_or(chrono::Utc::now());
    
    // Parse timezone from global settings or use default GMT+7
    let timezone_str = global_settings.timezone.as_deref().unwrap_or("Asia/Ho_Chi_Minh");
    
    // Map common timezone names to IANA identifiers (matching SignatureRenderer)
    let timezone_mapped = match timezone_str {
        "Midway Island" => "Pacific/Midway",
        "Hawaii" => "Pacific/Honolulu",
        "Alaska" => "America/Anchorage",
        "Pacific" => "America/Los_Angeles",
        "Mountain" => "America/Denver",
        "Central" => "America/Chicago",
        "Eastern" => "America/New_York",
        "Atlantic" => "America/Halifax",
        "Newfoundland" => "America/St_Johns",
        "London" => "Europe/London",
        "Berlin" => "Europe/Berlin",
        "Paris" => "Europe/Paris",
        "Rome" => "Europe/Rome",
        "Moscow" => "Europe/Moscow",
        "Tokyo" => "Asia/Tokyo",
        "Shanghai" => "Asia/Shanghai",
        "Hong Kong" => "Asia/Hong_Kong",
        "Singapore" => "Asia/Singapore",
        "Sydney" => "Australia/Sydney",
        "UTC" => "UTC",
        _ => timezone_str,
    };
    
    // Parse timezone offset (simplified approach for common timezones)
    let timezone_offset_hours = match timezone_mapped {
        "Asia/Ho_Chi_Minh" => 7,
        "Pacific/Midway" => -11,
        "Pacific/Honolulu" => -10,
        "America/Anchorage" => -9,
        "America/Los_Angeles" => -8,
        "America/Denver" => -7,
        "America/Chicago" => -6,
        "America/New_York" => -5,
        "America/Halifax" => -4,
        "Europe/London" => 0,
        "Europe/Berlin" | "Europe/Paris" | "Europe/Rome" => 1,
        "Europe/Moscow" => 3,
        "Asia/Tokyo" => 9,
        "Asia/Shanghai" | "Asia/Hong_Kong" | "Asia/Singapore" => 8,
        "Australia/Sydney" => 10,
        "UTC" => 0,
        _ => 7, // Default to GMT+7
    };
    
    let timezone_offset = chrono::FixedOffset::east_opt(timezone_offset_hours * 3600).unwrap();
    let signed_at_formatted = signed_at.with_timezone(&timezone_offset);
    
    // Format date according to locale (simplified)
    let locale = global_settings.locale.as_deref().unwrap_or("vi-VN");
    let date_str = if locale.starts_with("vi") {
        // Vietnamese format: DD/MM/YYYY, HH:MM:SS
        signed_at_formatted.format("%d/%m/%Y, %H:%M:%S").to_string()
    } else {
        // English/Default format: MM/DD/YYYY, HH:MM:SS
        signed_at_formatted.format("%m/%d/%Y, %H:%M:%S").to_string()
    };
    
    let mut signature_info_parts = Vec::new();
    
    // Always show reason first if require_signing_reason is enabled and reason exists
    if global_settings.require_signing_reason && !reason.is_empty() {
        signature_info_parts.push(format!("Reason: {}", reason));
    }
    
    // Show ID, email, and date if add_signature_id_to_the_documents is enabled
    if global_settings.add_signature_id_to_the_documents {
        signature_info_parts.push(format!("ID: {}", signature_id));
        signature_info_parts.push(signer_email.clone());
        signature_info_parts.push(date_str);
    }
    
    // If nothing to show, return early
    if signature_info_parts.is_empty() {
        return Ok(());
    }

    // Calculate text height dynamically (matching SignatureRenderer.tsx)
    let text_height = calculate_signature_text_height(
        global_settings,
        Some(submitter.id),
        &signer_email,
        reason
    );

    // Position the signature info at the BOTTOM of the field
    // Matching SignatureRenderer.tsx: text starts from bottom and goes up
    let info_x = x_pos + 5.0; // Match frontend padding of 5px
    let font_size = 8.0; // Match frontend font size
    let line_height = 10.0; // Match frontend line height
    
    // Text area is at the bottom: from pdf_y to (pdf_y + text_height)
    // Calculate actual text height needed
    let actual_text_height = (signature_info_parts.len() as f64 - 1.0) * line_height + font_size + 3.0;
    
    // Start rendering from the bottom of text area
    // First line should be at pdf_y + 3 (bottom padding), last line at the top
    let text_start_y = pdf_y + 3.0; // Bottom padding: 3px from the bottom
    
    // Create text content stream for signature info with multiple lines
    let mut text_operations = vec![
        Operation::new("BT", vec![]), // Begin text
        Operation::new("Tf", vec![
            Object::Name(b"Helvetica".to_vec()),
            Object::Real(font_size as f32),
        ]), // Set font
        Operation::new("rg", vec![

            Object::Real(0.0),
            Object::Real(0.0),
            Object::Real(0.0),
        ]), // Set text color to black
    ];
    
    // Render each line from bottom to top (matching SignatureRenderer.tsx)
    // SignatureRenderer draws: for (let i = textToShow.length - 1; i >= 0; i--)
    let num_lines = signature_info_parts.len();
    for (idx, line) in signature_info_parts.iter().enumerate() {
        // Calculate Y position: start from bottom and go up
        // Line 0 (first in array) at bottom, line N-1 (last) at top
        let line_y = text_start_y + ((num_lines - 1 - idx) as f64 * line_height);
        
        // Use Tm (text matrix) to set absolute position for each line
        text_operations.push(Operation::new("Tm", vec![
            Object::Real(1.0), // a: horizontal scaling
            Object::Real(0.0), // b: horizontal skewing
            Object::Real(0.0), // c: vertical skewing
            Object::Real(1.0), // d: vertical scaling
            Object::Real(info_x as f32), // e: horizontal position
            Object::Real(line_y as f32),  // f: vertical position
        ]));
        
        text_operations.push(Operation::new("Tj", vec![
            Object::string_literal(line.clone()),
        ])); // Show text
    }
    
    text_operations.push(Operation::new("ET", vec![])); // End text
    
    let content = Content { operations: text_operations };
    let content_data = content.encode()?;
    
    // Create a new content stream
    let stream = Stream::new(Dictionary::new(), content_data);
    let stream_id = doc.add_object(stream);
    
    // Get the page object and add stream to it
    if let Ok(page_obj) = doc.get_object_mut(page_id) {
        if let Ok(page_dict) = page_obj.as_dict_mut() {
            // Add to page's content array
            if let Ok(contents_obj) = page_dict.get_mut(b"Contents") {
                match contents_obj {
                    Object::Reference(_ref_id) => {
                        // For simplicity, replace the content reference with our new stream
                        *contents_obj = Object::Reference(stream_id);
                    },
                    Object::Array(ref mut contents_array) => {
                        contents_array.push(Object::Reference(stream_id));
                    },
                    _ => {
                        // Replace with new content stream
                        *contents_obj = Object::Reference(stream_id);
                    }
                }
            } else {
                // Add new Contents array
                page_dict.set(b"Contents", Object::Reference(stream_id));
            }
        }
    }
    
    Ok(())
}

// Helper function to extract filename from URL
fn extract_filename_from_url(url: &str) -> String {
    url.split('/').last().unwrap_or("file").to_string()
}

// Helper function to generate hash ID similar to frontend hashId function
fn hash_id(value: i64) -> String {
    let str_value = value.to_string();
    
    // Create 32-bit hash from value
    let mut hash: i32 = 0;
    for ch in str_value.chars() {
        hash = ((hash << 5).wrapping_sub(hash).wrapping_add(ch as i32)) | 0;
    }
    
    // Generate hex string (8 characters from 32-bit hash)
    let mut hex = String::new();
    for i in 0..8 {
        let h = ((hash >> (i * 4)) & 0xF) as u8;
        hex.push_str(&format!("{:X}", h));
    }
    
    // Repeat to get 32 characters: hex.len() = 8, we need 32, so repeat 4 times
    let hex32 = format!("{}{}{}{}", hex, hex, hex, hex);
    
    // Format as UUID (8-4-4-4-12 = 32 characters)
    format!(
        "{}-{}-{}-{}-{}",
        &hex32[0..8],
        &hex32[8..12],
        &hex32[12..16],
        &hex32[16..20],
        &hex32[20..32]
    )
}

// Calculate text height for signature info (matching SignatureRenderer.tsx logic)
fn calculate_signature_text_height(
    global_settings: &crate::database::models::DbGlobalSettings,
    submitter_id: Option<i64>,
    submitter_email: &str,
    reason: &str,
) -> f64 {
    let mut line_count = 0;
    
    if global_settings.add_signature_id_to_the_documents {
        if submitter_id.is_some() { line_count += 1; }
        if !submitter_email.is_empty() { line_count += 1; }
        line_count += 1; // date
    }
    
    if global_settings.require_signing_reason && !reason.is_empty() {
        line_count += 1;
    }
    
    // Match SignatureRenderer.tsx: (lineCount - 1) * 10 + 8 + 3
    if line_count > 0 {
        ((line_count - 1) as f64 * 10.0) + 8.0 + 3.0
    } else {
        0.0
    }
}

// Render signature ID information below the signature
fn render_signature_id_info(
    doc: &mut lopdf::Document,
    page_id: lopdf::ObjectId,
    submitter: &crate::database::models::DbSubmitter,
    signature_data: &serde_json::Value,
    x_pos: f64,
    pdf_y: f64,
    field_width: f64,
    field_height: f64,
    global_settings: &crate::database::models::DbGlobalSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    use lopdf::{Object, Stream, Dictionary};
    use lopdf::content::{Content, Operation};

    // Generate signature ID using hashId function (matching frontend)
    let signature_id = hash_id(submitter.id + 1);

    // Get reason from signature data
    let reason = signature_data.get("reason")
        .and_then(|r| r.as_str())
        .unwrap_or("");

    // Format the signature information
    let signer_email = submitter.email.clone();
    let signed_at = submitter.signed_at.unwrap_or(chrono::Utc::now());
    
    // Parse timezone from global settings or use default GMT+7
    let timezone_str = global_settings.timezone.as_deref().unwrap_or("Asia/Ho_Chi_Minh");
    
    // Map common timezone names to IANA identifiers (matching SignatureRenderer)
    let timezone_mapped = match timezone_str {
        "Midway Island" => "Pacific/Midway",
        "Hawaii" => "Pacific/Honolulu",
        "Alaska" => "America/Anchorage",
        "Pacific" => "America/Los_Angeles",
        "Mountain" => "America/Denver",
        "Central" => "America/Chicago",
        "Eastern" => "America/New_York",
        "Atlantic" => "America/Halifax",
        "Newfoundland" => "America/St_Johns",
        "London" => "Europe/London",
        "Berlin" => "Europe/Berlin",
        "Paris" => "Europe/Paris",
        "Rome" => "Europe/Rome",
        "Moscow" => "Europe/Moscow",
        "Tokyo" => "Asia/Tokyo",
        "Shanghai" => "Asia/Shanghai",
        "Hong Kong" => "Asia/Hong_Kong",
        "Singapore" => "Asia/Singapore",
        "Sydney" => "Australia/Sydney",
        "UTC" => "UTC",
        _ => timezone_str,
    };
    
    // Parse timezone offset (simplified approach for common timezones)
    let timezone_offset_hours = match timezone_mapped {
        "Asia/Ho_Chi_Minh" => 7,
        "Pacific/Midway" => -11,
        "Pacific/Honolulu" => -10,
        "America/Anchorage" => -9,
        "America/Los_Angeles" => -8,
        "America/Denver" => -7,
        "America/Chicago" => -6,
        "America/New_York" => -5,
        "America/Halifax" => -4,
        "Europe/London" => 0,
        "Europe/Berlin" | "Europe/Paris" | "Europe/Rome" => 1,
        "Europe/Moscow" => 3,
        "Asia/Tokyo" => 9,
        "Asia/Shanghai" | "Asia/Hong_Kong" | "Asia/Singapore" => 8,
        "Australia/Sydney" => 10,
        "UTC" => 0,
        _ => 7, // Default to GMT+7
    };
    
    let timezone_offset = chrono::FixedOffset::east_opt(timezone_offset_hours * 3600).unwrap();
    let signed_at_formatted = signed_at.with_timezone(&timezone_offset);
    
    // Format date according to locale (simplified)
    let locale = global_settings.locale.as_deref().unwrap_or("vi-VN");
    let date_str = if locale.starts_with("vi") {
        // Vietnamese format: DD/MM/YYYY, HH:MM:SS
        signed_at_formatted.format("%d/%m/%Y, %H:%M:%S").to_string()
    } else {
        // English/Default format: MM/DD/YYYY, HH:MM:SS
        signed_at_formatted.format("%m/%d/%Y, %H:%M:%S").to_string()
    };
    
    let mut signature_info_parts = Vec::new();
    
    // Always show reason first if require_signing_reason is enabled and reason exists
    if global_settings.require_signing_reason && !reason.is_empty() {
        signature_info_parts.push(format!("Reason: {}", reason));
    }
    
    // Show ID, email, and date if add_signature_id_to_the_documents is enabled
    if global_settings.add_signature_id_to_the_documents {
        signature_info_parts.push(format!("ID: {}", signature_id));
        signature_info_parts.push(signer_email.clone());
        signature_info_parts.push(date_str);
    }
    
    // If nothing to show, return early
    if signature_info_parts.is_empty() {
        return Ok(());
    }

    // Calculate text height dynamically (matching SignatureRenderer.tsx)
    let text_height = calculate_signature_text_height(
        global_settings,
        Some(submitter.id),
        &signer_email,
        reason
    );

    // Position the signature info at the BOTTOM of the field
    // Matching SignatureRenderer.tsx: text starts from bottom and goes up
    let info_x = x_pos + 5.0; // Match frontend padding of 5px
    let font_size = 8.0; // Match frontend font size
    let line_height = 10.0; // Match frontend line height
    
    // Text area is at the bottom: from pdf_y to (pdf_y + text_height)
    // Calculate actual text height needed
    let actual_text_height = (signature_info_parts.len() as f64 - 1.0) * line_height + font_size + 3.0;
    
    // Start rendering from the bottom of text area
    // First line should be at pdf_y + 3 (bottom padding), last line at the top
    let text_start_y = pdf_y + 3.0; // Bottom padding: 3px from the bottom
    
    // Create text content stream for signature info with multiple lines
    let mut text_operations = vec![
        Operation::new("BT", vec![]), // Begin text
        Operation::new("Tf", vec![
            Object::Name(b"Helvetica".to_vec()),
            Object::Real(font_size as f32),
        ]), // Set font
        Operation::new("rg", vec![
            Object::Real(0.0),
            Object::Real(0.0),
            Object::Real(0.0),
        ]), // Set text color to black
    ];
    
    // Render each line from bottom to top (matching SignatureRenderer.tsx)
    // SignatureRenderer draws: for (let i = textToShow.length - 1; i >= 0; i--)
    let num_lines = signature_info_parts.len();
    for (idx, line) in signature_info_parts.iter().enumerate() {
        // Calculate Y position: start from bottom and go up
        // Line 0 (first in array) at bottom, line N-1 (last) at top
        let line_y = text_start_y + ((num_lines - 1 - idx) as f64 * line_height);
        
        // Use Tm (text matrix) to set absolute position for each line
        text_operations.push(Operation::new("Tm", vec![
            Object::Real(1.0), // a: horizontal scaling
            Object::Real(0.0), // b: horizontal skewing
            Object::Real(0.0), // c: vertical skewing
            Object::Real(1.0), // d: vertical scaling
            Object::Real(info_x as f32), // e: horizontal position
            Object::Real(line_y as f32),  // f: vertical position
        ]));
        
        text_operations.push(Operation::new("Tj", vec![
            Object::string_literal(line.clone()),
        ])); // Show text
    }
    
    text_operations.push(Operation::new("ET", vec![])); // End text
    
    let content = Content { operations: text_operations };
    let content_data = content.encode()?;
    
    // Create a new content stream
    let stream = Stream::new(Dictionary::new(), content_data);
    let stream_id = doc.add_object(stream);
    
    // Get the page object and add stream to it
    if let Ok(page_obj) = doc.get_object_mut(page_id) {
        if let Ok(page_dict) = page_obj.as_dict_mut() {
            // Add to page's content array
            if let Ok(contents_obj) = page_dict.get_mut(b"Contents") {
                match contents_obj {
                    Object::Reference(_ref_id) => {
                        // For simplicity, replace the content reference with our new stream
                        *contents_obj = Object::Reference(stream_id);
                    },
                    Object::Array(ref mut contents_array) => {
                        contents_array.push(Object::Reference(stream_id));
                    },
                    _ => {
                        // Replace with new content stream
                        *contents_obj = Object::Reference(stream_id);
                    }
                }
            } else {
                // Add new Contents array
                page_dict.set(b"Contents", Object::Reference(stream_id));
            }
        }
    }
    
    Ok(())
}

// Helper function to extract filename from URL
fn extract_filename_from_url(url: &str) -> String {
    url.split('/').last().unwrap_or("file").to_string()
}

// Helper function to generate hash ID similar to frontend hashId function
fn hash_id(value: i64) -> String {
    let str_value = value.to_string();
    
    // Create 32-bit hash from value
    let mut hash: i32 = 0;
    for ch in str_value.chars() {
        hash = ((hash << 5).wrapping_sub(hash).wrapping_add(ch as i32)) | 0;
    }
    
    // Generate hex string (8 characters from 32-bit hash)
    let mut hex = String::new();
    for i in 0..8 {
        let h = ((hash >> (i * 4)) & 0xF) as u8;
        hex.push_str(&format!("{:X}", h));
    }
    
    // Repeat to get 32 characters: hex.len() = 8, we need 32, so repeat 4 times
    let hex32 = format!("{}{}{}{}", hex, hex, hex, hex);
    
    // Format as UUID (8-4-4-4-12 = 32 characters)
    format!(
        "{}-{}-{}-{}-{}",
        &hex32[0..8],
        &hex32[8..12],
        &hex32[12..16],
        &hex32[16..20],
        &hex32[20..32]
    )
}

// Calculate text height for signature info (matching SignatureRenderer.tsx logic)
fn calculate_signature_text_height(
    global_settings: &crate::database::models::DbGlobalSettings,
    submitter_id: Option<i64>,
    submitter_email: &str,
    reason: &str,
) -> f64 {
    let mut line_count = 0;
    
    if global_settings.add_signature_id_to_the_documents {
        if submitter_id.is_some() { line_count += 1; }
        if !submitter_email.is_empty() { line_count += 1; }
        line_count += 1; // date
    }
    
    if global_settings.require_signing_reason && !reason.is_empty() {
        line_count += 1;
    }
    
    // Match SignatureRenderer.tsx: (lineCount - 1) * 10 + 8 + 3
    if line_count > 0 {
        ((line_count - 1) as f64 * 10.0) + 8.0 + 3.0
    } else {
        0.0
    }
}

// Render signature ID information below the signature
fn render_signature_id_info(
    doc: &mut lopdf::Document,
    page_id: lopdf::ObjectId,
    submitter: &crate::database::models::DbSubmitter,
    signature_data: &serde_json::Value,
    x_pos: f64,
    pdf_y: f64,
    field_width: f64,
    field_height: f64,
    global_settings: &crate::database::models::DbGlobalSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    use lopdf::{Object, Stream, Dictionary};
    use lopdf::content::{Content, Operation};

    // Generate signature ID using hashId function (matching frontend)
    let signature_id = hash_id(submitter.id + 1);

    // Get reason from signature data
    let reason = signature_data.get("reason")
        .and_then(|r| r.as_str())
        .unwrap_or("");

    // Format the signature information
    let signer_email = submitter.email.clone();
    let signed_at = submitter.signed_at.unwrap_or(chrono::Utc::now());
    
    // Parse timezone from global settings or use default GMT+7
    let timezone_str = global_settings.timezone.as_deref().unwrap_or("Asia/Ho_Chi_Minh");
    
    // Map common timezone names to IANA identifiers (matching SignatureRenderer)
    let timezone_mapped = match timezone_str {
        "Midway Island" => "Pacific/Midway",
        "Hawaii" => "Pacific/Honolulu",
        "Alaska" => "America/Anchorage",
        "Pacific" => "America/Los_Angeles",
        "Mountain" => "America/Denver",
        "Central" => "America/Chicago",
        "Eastern" => "America/New_York",
        "Atlantic" => "America/Halifax",
        "Newfoundland" => "America/St_Johns",
        "London" => "Europe/London",
        "Berlin" => "Europe/Berlin",
        "Paris" => "Europe/Paris",
        "Rome" => "Europe/Rome",
        "Moscow" => "Europe/Moscow",
        "Tokyo" => "Asia/Tokyo",
        "Shanghai" => "Asia/Shanghai",
        "Hong Kong" => "Asia/Hong_Kong",
        "Singapore" => "Asia/Singapore",
        "Sydney" => "Australia/Sydney",
        "UTC" => "UTC",
        _ => timezone_str,
    };
    
    // Parse timezone offset (simplified approach for common timezones)
    let timezone_offset_hours = match timezone_mapped {
        "Asia/Ho_Chi_Minh" => 7,
        "Pacific/Midway" => -11,
        "Pacific/Honolulu" => -10,
        "America/Anchorage" => -9,
        "America/Los_Angeles" => -8,
        "America/Denver" => -7,
        "America/Chicago" => -6,
        "America/New_York" => -5,
        "America/Halifax" => -4,
        "Europe/London" => 0,
        "Europe/Berlin" | "Europe/Paris" | "Europe/Rome" => 1,
        "Europe/Moscow" => 3,
        "Asia/Tokyo" => 9,
        "Asia/Shanghai" | "Asia/Hong_Kong" | "Asia/Singapore" => 8,
        "Australia/Sydney" => 10,
        "UTC" => 0,
        _ => 7, // Default to GMT+7
    };
    
    let timezone_offset = chrono::FixedOffset::east_opt(timezone_offset_hours * 3600).unwrap();
    let signed_at_formatted = signed_at.with_timezone(&timezone_offset);
    
    // Format date according to locale (simplified)
    let locale = global_settings.locale.as_deref().unwrap_or("vi-VN");
    let date_str = if locale.starts_with("vi") {
        // Vietnamese format: DD/MM/YYYY, HH:MM:SS
        signed_at_formatted.format("%d/%m/%Y, %H:%M:%S").to_string()
    } else {
        // English/Default format: MM/DD/YYYY, HH:MM:SS
        signed_at_formatted.format("%m/%d/%Y, %H:%M:%S").to_string()
    };
    
    let mut signature_info_parts = Vec::new();
    
    // Always show reason first if require_signing_reason is enabled and reason exists
    if global_settings.require_signing_reason && !reason.is_empty() {
        signature_info_parts.push(format!("Reason: {}", reason));
    }
    
    // Show ID, email, and date if add_signature_id_to_the_documents is enabled
    if global_settings.add_signature_id_to_the_documents {
        signature_info_parts.push(format!("ID: {}", signature_id));
        signature_info_parts.push(signer_email.clone());
        signature_info_parts.push(date_str);
    }
    
    // If nothing to show, return early
    if signature_info_parts.is_empty() {
        return Ok(());
    }

    // Calculate text height dynamically (matching SignatureRenderer.tsx)
    let text_height = calculate_signature_text_height(
        global_settings,
        Some(submitter.id),
        &signer_email,
        reason
    );

    // Position the signature info at the BOTTOM of the field
    // Matching SignatureRenderer.tsx: text starts from bottom and goes up
    let info_x = x_pos + 5.0; // Match frontend padding of 5px
    let font_size = 8.0; // Match frontend font size
    let line_height = 10.0; // Match frontend line height
    
    // Text area is at the bottom: from pdf_y to (pdf_y + text_height)
    // Calculate actual text height needed
    let actual_text_height = (signature_info_parts.len() as f64 - 1.0) * line_height + font_size + 3.0;
    
    // Start rendering from the bottom of text area
    // First line should be at pdf_y + 3 (bottom padding), last line at the top
    let text_start_y = pdf_y + 3.0; // Bottom padding: 3px from the bottom
    
    // Create text content stream for signature info with multiple lines
    let mut text_operations = vec![
        Operation::new("BT", vec![]), // Begin text
        Operation::new("Tf", vec![
            Object::Name(b"Helvetica".to_vec()),
            Object::Real(font_size as f32),
        ]), // Set font
        Operation::new("rg", vec![
            Object::Real(0.0),
            Object::Real(0.0),
            Object::Real(0.0),
        ]), // Set text color to black
    ];
    
    // Render each line from bottom to top (matching SignatureRenderer.tsx)
    // SignatureRenderer draws: for (let i = textToShow.length - 1; i >= 0; i--)
    let num_lines = signature_info_parts.len();
    for (idx, line) in signature_info_parts.iter().enumerate() {
        // Calculate Y position: start from bottom and go up
        // Line 0 (first in array) at bottom, line N-1 (last) at top
        let line_y = text_start_y + ((num_lines - 1 - idx) as f64 * line_height);
        
        // Use Tm (text matrix) to set absolute position for each line
        text_operations.push(Operation::new("Tm", vec![
            Object::Real(1.0), // a: horizontal scaling
            Object::Real(0.0), // b: horizontal skewing
            Object::Real(0.0), // c: vertical skewing
            Object::Real(1.0), // d: vertical scaling
            Object::Real(info_x as f32), // e: horizontal position
            Object::Real(line_y as f32),  // f: vertical position
        ]));
        
        text_operations.push(Operation::new("Tj", vec![
            Object::string_literal(line.clone()),
        ])); // Show text
    }
    
    text_operations.push(Operation::new("ET", vec![])); // End text
    
    let content = Content { operations: text_operations };
    let content_data = content.encode()?;
    
    // Create a new content stream
    let stream = Stream::new(Dictionary::new(), content_data);
    let stream_id = doc.add_object(stream);
    
    // Get the page object and add stream to it
    if let Ok(page_obj) = doc.get_object_mut(page_id) {
        if let Ok(page_dict) = page_obj.as_dict_mut() {
            // Add to page's content array
            if let Ok(contents_obj) = page_dict.get_mut(b"Contents") {
                match contents_obj {
                    Object::Reference(_ref_id) => {
                        // For simplicity, replace the content reference with our new stream
                        *contents_obj = Object::Reference(stream_id);
                    },
                    Object::Array(ref mut contents_array) => {
                        contents_array.push(Object::Reference(stream_id));
                    },
                    _ => {
                        // Replace with new content stream
                        *contents_obj = Object::Reference(stream_id);
                    }
                }
            } else {
                // Add new Contents array
                page_dict.set(b"Contents", Object::Reference(stream_id));
            }
        }
    }
    
    Ok(())
}

// Helper function to extract filename from URL
fn extract_filename_from_url(url: &str) -> String {
    url.split('/').last().unwrap_or("file").to_string()
}

// Helper function to generate hash ID similar to frontend hashId function
fn hash_id(value: i64) -> String {
    let str_value = value.to_string();
    
    // Create 32-bit hash from value
    let mut hash: i32 = 0;
    for ch in str_value.chars() {
        hash = ((hash << 5).wrapping_sub(hash).wrapping_add(ch as i32)) | 0;
    }
    
    // Generate hex string (8 characters from 32-bit hash)
    let mut hex = String::new();
    for i in 0..8 {
        let h = ((hash >> (i * 4)) & 0xF) as u8;
        hex.push_str(&format!("{:X}", h));
    }
    
    // Repeat to get 32 characters: hex.len() = 8, we need 32, so repeat 4 times
    let hex32 = format!("{}{}{}{}", hex, hex, hex, hex);
    
    // Format as UUID (8-4-4-4-12 = 32 characters)
    format!(
        "{}-{}-{}-{}-{}",
        &hex32[0..8],
        &hex32[8..12],
        &hex32[12..16],
        &hex32[16..20],
        &hex32[20..32]
    )
}

// Calculate text height for signature info (matching SignatureRenderer.tsx logic)
fn calculate_signature_text_height(
    global_settings: &crate::database::models::DbGlobalSettings,
    submitter_id: Option<i64>,
    submitter_email: &str,
    reason: &str,
) -> f64 {
    let mut line_count = 0;
    
    if global_settings.add_signature_id_to_the_documents {
        if submitter_id.is_some() { line_count += 1; }
        if !submitter_email.is_empty() { line_count += 1; }
        line_count += 1; // date
    }
    
    if global_settings.require_signing_reason && !reason.is_empty() {
        line_count += 1;
    }
    
    // Match SignatureRenderer.tsx: (lineCount - 1) * 10 + 8 + 3
    if line_count > 0 {
        ((line_count - 1) as f64 * 10.0) + 8.0 + 3.0
    } else {
        0.0
    }
}

// Render signature ID information below the signature
fn render_signature_id_info(
    doc: &mut lopdf::Document,
    page_id: lopdf::ObjectId,
    submitter: &crate::database::models::DbSubmitter,
    signature_data: &serde_json::Value,
    x_pos: f64,
    pdf_y: f64,
    field_width: f64,
    field_height: f64,
    global_settings: &crate::database::models::DbGlobalSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    use lopdf::{Object, Stream, Dictionary};
    use lopdf::content::{Content, Operation};

    // Generate signature ID using hashId function (matching frontend)
    let signature_id = hash_id(submitter.id + 1);

    // Get reason from signature data
    let reason = signature_data.get("reason")
        .and_then(|r| r.as_str())
        .unwrap_or("");

    // Format the signature information
    let signer_email = submitter.email.clone();
    let signed_at = submitter.signed_at.unwrap_or(chrono::Utc::now());
    
    // Parse timezone from global settings or use default GMT+7
    let timezone_str = global_settings.timezone.as_deref().unwrap_or("Asia/Ho_Chi_Minh");
    
    // Map common timezone names to IANA identifiers (matching SignatureRenderer)
    let timezone_mapped = match timezone_str {
        "Midway Island" => "Pacific/Midway",
        "Hawaii" => "Pacific/Honolulu",
        "Alaska" => "America/Anchorage",
        "Pacific" => "America/Los_Angeles",
        "Mountain" => "America/Denver",
        "Central" => "America/Chicago",
        "Eastern" => "America/New_York",
        "Atlantic" => "America/Halifax",
        "Newfoundland" => "America/St_Johns",
        "London" => "Europe/London",
        "Berlin" => "Europe/Berlin",
        "Paris" => "Europe/Paris",
        "Rome" => "Europe/Rome",
        "Moscow" => "Europe/Moscow",
        "Tokyo" => "Asia/Tokyo",
        "Shanghai" => "Asia/Shanghai",
        "Hong Kong" => "Asia/Hong_Kong",
        "Singapore" => "Asia/Singapore",
        "Sydney" => "Australia/Sydney",
        "UTC" => "UTC",
        _ => timezone_str,
    };
    
    // Parse timezone offset (simplified approach for common timezones)
    let timezone_offset_hours = match timezone_mapped {
        "Asia/Ho_Chi_Minh" => 7,
        "Pacific/Midway" => -11,
        "Pacific/Honolulu" => -10,
        "America/Anchorage" => -9,
        "America/Los_Angeles" => -8,
        "America/Denver" => -7,
        "America/Chicago" => -6,
        "America/New_York" => -5,
        "America/Halifax" => -4,
        "Europe/London" => 0,
        "Europe/Berlin" | "Europe/Paris" | "Europe/Rome" => 1,
        "Europe/Moscow" => 3,
        "Asia/Tokyo" => 9,
        "Asia/Shanghai" | "Asia/Hong_Kong" | "Asia/Singapore" => 8,
        "Australia/Sydney" => 10,
        "UTC" => 0,
        _ => 7, // Default to GMT+7
    };
    
    let timezone_offset = chrono::FixedOffset::east_opt(timezone_offset_hours * 3600).unwrap();
    let signed_at_formatted = signed_at.with_timezone(&timezone_offset);
    
    // Format date according to locale (simplified)
    let locale = global_settings.locale.as_deref().unwrap_or("vi-VN");
    let date_str = if locale.starts_with("vi") {
        // Vietnamese format: DD/MM/YYYY, HH:MM:SS
        signed_at_formatted.format("%d/%m/%Y, %H:%M:%S").to_string()
    } else {
        // English/Default format: MM/DD/YYYY, HH:MM:SS
        signed_at_formatted.format("%m/%d/%Y, %H:%M:%S").to_string()
    };
    
    let mut signature_info_parts = Vec::new();
    
    // Always show reason first if require_signing_reason is enabled and reason exists
    if global_settings.require_signing_reason && !reason.is_empty() {
        signature_info_parts.push(format!("Reason: {}", reason));
    }
    
    // Show ID, email, and date if add_signature_id_to_the_documents is enabled
    if global_settings.add_signature_id_to_the_documents {
        signature_info_parts.push(format!("ID: {}", signature_id));
        signature_info_parts.push(signer_email.clone());
        signature_info_parts.push(date_str);
    }
    
    // If nothing to show, return early
    if signature_info_parts.is_empty() {
        return Ok(());
    }

    // Calculate text height dynamically (matching SignatureRenderer.tsx)
    let text_height = calculate_signature_text_height(
        global_settings,
        Some(submitter.id),
        &signer_email,
        reason
    );

    // Position the signature info at the BOTTOM of the field
    // Matching SignatureRenderer.tsx: text starts from bottom and goes up
    let info_x = x_pos + 5.0; // Match frontend padding of 5px
    let font_size = 8.0; // Match frontend font size
    let line_height = 10.0; // Match frontend line height
    
    // Text area is at the bottom: from pdf_y to (pdf_y + text_height)
    // Calculate actual text height needed
    let actual_text_height = (signature_info_parts.len() as f64 - 1.0) * line_height + font_size + 3.0;
    
    // Start rendering from the bottom of text area
    // First line should be at pdf_y + 3 (bottom padding), last line at the top
    let text_start_y = pdf_y + 3.0; // Bottom padding: 3px from the bottom
    
    // Create text content stream for signature info with multiple lines
    let mut text_operations = vec![
        Operation::new("BT", vec![]), // Begin text
        Operation::new("Tf", vec![
            Object::Name(b"Helvetica".to_vec()),
            Object::Real(font_size as f32),
        ]), // Set font
        Operation::new("rg", vec![
            Object::Real(0.0),
            Object::Real(0.0),
            Object::Real(0.0),
        ]), // Set text color to black
    ];
    
    // Render each line from bottom to top (matching SignatureRenderer.tsx)
    // SignatureRenderer draws: for (let i = textToShow.length - 1; i >= 0; i--)
    let num_lines = signature_info_parts.len();
    for (idx, line) in signature_info_parts.iter().enumerate() {
        // Calculate Y position: start from bottom and go up
        // Line 0 (first in array) at bottom, line N-1 (last) at top
        let line_y = text_start_y + ((num_lines - 1 - idx) as f64 * line_height);
        
        // Use Tm (text matrix) to set absolute position for each line
        text_operations.push(Operation::new("Tm", vec![
            Object::Real(1.0), // a: horizontal scaling
            Object::Real(0.0), // b: horizontal skewing
            Object::Real(0.0), // c: vertical skewing
            Object::Real(1.0), // d: vertical scaling
            Object::Real(info_x as f32), // e: horizontal position
            Object::Real(line_y as f32),  // f: vertical position
        ]));
        
        text_operations.push(Operation::new("Tj", vec![
            Object::string_literal(line.clone()),
        ])); // Show text
    }
    
    text_operations.push(Operation::new("ET", vec![])); // End text
    
    let content = Content { operations: text_operations };
    let content_data = content.encode()?;
    
    // Create a new content stream
    let stream = Stream::new(Dictionary::new(), content_data);
    let stream_id = doc.add_object(stream);
    
    // Get the page object and add stream to it
    if let Ok(page_obj) = doc.get_object_mut(page_id) {
        if let Ok(page_dict) = page_obj.as_dict_mut() {
            // Add to page's content array
            if let Ok(contents_obj) = page_dict.get_mut(b"Contents") {
                match contents_obj {
                    Object::Reference(_ref_id) => {
                        // For simplicity, replace the content reference with our new stream
                        *contents_obj = Object::Reference(stream_id);
                    },
                    Object::Array(ref mut contents_array) => {
                        contents_array.push(Object::Reference(stream_id));
                    },
                    _ => {
                        // Replace with new content stream
                        *contents_obj = Object::Reference(stream_id);
                    }
                }
            } else {
                // Add new Contents array
                page_dict.set(b"Contents", Object::Reference(stream_id));
            }
        }
    }
    
    Ok(())
}

// Helper function to extract filename from URL
fn extract_filename_from_url(url: &str) -> String {
    url.split('/').last().unwrap_or("file").to_string()
}

// Helper function to generate hash ID similar to frontend hashId function
fn hash_id(value: i64) -> String {
    let str_value = value.to_string();
    
    // Create 32-bit hash from value
    let mut hash: i32 = 0;
    for ch in str_value.chars() {
        hash = ((hash << 5).wrapping_sub(hash).wrapping_add(ch as i32)) | 0;
    }
    
    // Generate hex string (8 characters from 32-bit hash)
    let mut hex = String::new();
    for i in 0..8 {
        let h = ((hash >> (i * 4)) & 0xF) as u8;
        hex.push_str(&format!("{:X}", h));
    }
    
    // Repeat to get 32 characters: hex.len() = 8, we need 32, so repeat 4 times
    let hex32 = format!("{}{}{}{}", hex, hex, hex, hex);
    
    // Format as UUID (8-4-4-4-12 = 32 characters)
    format!(
        "{}-{}-{}-{}-{}",
        &hex32[0..8],
        &hex32[8..12],
        &hex32[12..16],
        &hex32[16..20],
        &hex32[20..32]
    )
}

// Calculate text height for signature info (matching SignatureRenderer.tsx logic)
fn calculate_signature_text_height(
    global_settings: &crate::database::models::DbGlobalSettings,
    submitter_id: Option<i64>,
    submitter_email: &str,
    reason: &str,
) -> f64 {
    let mut line_count = 0;
    
    if global_settings.add_signature_id_to_the_documents {
        if submitter_id.is_some() { line_count += 1; }
        if !submitter_email.is_empty() { line_count += 1; }
        line_count += 1; // date
    }
    
    if global_settings.require_signing_reason && !reason.is_empty() {
        line_count += 1;
    }
    
    // Match SignatureRenderer.tsx: (lineCount - 1) * 10 + 8 + 3
    if line_count > 0 {
        ((line_count - 1) as f64 * 10.0) + 8.0 + 3.0
    } else {
        0.0
    }
}

// Render signature ID information below the signature
fn render_signature_id_info(
    doc: &mut lopdf::Document,
    page_id: lopdf::ObjectId,
    submitter: &crate::database::models::DbSubmitter,
    signature_data: &serde_json::Value,
    x_pos: f64,
    pdf_y: f64,
    field_width: f64,
    field_height: f64,
    global_settings: &crate::database::models::DbGlobalSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    use lopdf::{Object, Stream, Dictionary};
    use lopdf::content::{Content, Operation};

    // Generate signature ID using hashId function (matching frontend)
    let signature_id = hash_id(submitter.id + 1);

    // Get reason from signature data
    let reason = signature_data.get("reason")
        .and_then(|r| r.as_str())
        .unwrap_or("");

    // Format the signature information
    let signer_email = submitter.email.clone();
    let signed_at = submitter.signed_at.unwrap_or(chrono::Utc::now());
    
    // Parse timezone from global settings or use default GMT+7
    let timezone_str = global_settings.timezone.as_deref().unwrap_or("Asia/Ho_Chi_Minh");
    
    // Map common timezone names to IANA identifiers (matching SignatureRenderer)
    let timezone_mapped = match timezone_str {
        "Midway Island" => "Pacific/Midway",
        "Hawaii" => "Pacific/Honolulu",
        "Alaska" => "America/Anchorage",
        "Pacific" => "America/Los_Angeles",
        "Mountain" => "America/Denver",
        "Central" => "America/Chicago",
        "Eastern" => "America/New_York",
        "Atlantic" => "America/Halifax",
        "Newfoundland" => "America/St_Johns",
        "London" => "Europe/London",
        "Berlin" => "Europe/Berlin",
        "Paris" => "Europe/Paris",
        "Rome" => "Europe/Rome",
        "Moscow" => "Europe/Moscow",
        "Tokyo" => "Asia/Tokyo",
        "Shanghai" => "Asia/Shanghai",
        "Hong Kong" => "Asia/Hong_Kong",
        "Singapore" => "Asia/Singapore",
        "Sydney" => "Australia/Sydney",
        "UTC" => "UTC",
        _ => timezone_str,
    };
    
    // Parse timezone offset (simplified approach for common timezones)
    let timezone_offset_hours = match timezone_mapped {
        "Asia/Ho_Chi_Minh" => 7,
        "Pacific/Midway" => -11,
        "Pacific/Honolulu" => -10,
        "America/Anchorage" => -9,
        "America/Los_Angeles" => -8,
        "America/Denver" => -7,
        "America/Chicago" => -6,
        "America/New_York" => -5,
        "America/Halifax" => -4,
        "Europe/London" => 0,
        "Europe/Berlin" | "Europe/Paris" | "Europe/Rome" => 1,
        "Europe/Moscow" => 3,
        "Asia/Tokyo" => 9,
        "Asia/Shanghai" | "Asia/Hong_Kong" | "Asia/Singapore" => 8,
        "Australia/Sydney" => 10,
        "UTC" => 0,
        _ => 7, // Default to GMT+7
    };
    
    let timezone_offset = chrono::FixedOffset::east_opt(timezone_offset_hours * 3600).unwrap();
    let signed_at_formatted = signed_at.with_timezone(&timezone_offset);
    
    // Format date according to locale (simplified)
    let locale = global_settings.locale.as_deref().unwrap_or("vi-VN");
    let date_str = if locale.starts_with("vi") {
        // Vietnamese format: DD/MM/YYYY, HH:MM:SS
        signed_at_formatted.format("%d/%m/%Y, %H:%M:%S").to_string()
    } else {
        // English/Default format: MM/DD/YYYY, HH:MM:SS
        signed_at_formatted.format("%m/%d/%Y, %H:%M:%S").to_string()
    };
    
    let mut signature_info_parts = Vec::new();
    
    // Always show reason first if require_signing_reason is enabled and reason exists
    if global_settings.require_signing_reason && !reason.is_empty() {
        signature_info_parts.push(format!("Reason: {}", reason));
    }
    
    // Show ID, email, and date if add_signature_id_to_the_documents is enabled
    if global_settings.add_signature_id_to_the_documents {
        signature_info_parts.push(format!("ID: {}", signature_id));
        signature_info_parts.push(signer_email.clone());
        signature_info_parts.push(date_str);
    }
    
    // If nothing to show, return early
    if signature_info_parts.is_empty() {
        return Ok(());
    }

    // Calculate text height dynamically (matching SignatureRenderer.tsx)
    let text_height = calculate_signature_text_height(
        global_settings,
        Some(submitter.id),
        &signer_email,
        reason
    );

    // Position the signature info at the BOTTOM of the field
    // Matching SignatureRenderer.tsx: text starts from bottom and goes up
    let info_x = x_pos + 5.0; // Match frontend padding of 5px
    let font_size = 8.0; // Match frontend font size
    let line_height = 10.0; // Match frontend line height
    
    // Text area is at the bottom: from pdf_y to (pdf_y + text_height)
    // Calculate actual text height needed
    let actual_text_height = (signature_info_parts.len() as f64 - 1.0) * line_height + font_size + 3.0;
    
    // Start rendering from the bottom of text area
    // First line should be at pdf_y + 3 (bottom padding), last line at the top
    let text_start_y = pdf_y + 3.0; // Bottom padding: 3px from the bottom
    
    // Create text content stream for signature info with multiple lines
    let mut text_operations = vec![
        Operation::new("BT", vec![]), // Begin text
        Operation::new("Tf", vec![
            Object::Name(b"Helvetica".to_vec()),
            Object::Real(font_size as f32),
        ]), // Set font
        Operation::new("rg", vec![
            Object::Real(0.0),
            Object::Real(0.0),
            Object::Real(0.0),
        ]), // Set text color to black
    ];
    
    // Render each line from bottom to top (matching SignatureRenderer.tsx)
    // SignatureRenderer draws: for (let i = textToShow.length - 1; i >= 0; i--)
    let num_lines = signature_info_parts.len();
    for (idx, line) in signature_info_parts.iter().enumerate() {
        // Calculate Y position: start from bottom and go up
        // Line 0 (first in array) at bottom, line N-1 (last) at top
        let line_y = text_start_y + ((num_lines - 1 - idx) as f64 * line_height);
        
        // Use Tm (text matrix) to set absolute position for each line
        text_operations.push(Operation::new("Tm", vec![
            Object::Real(1.0), // a: horizontal scaling
            Object::Real(0.0), // b: horizontal skewing
            Object::Real(0.0), // c: vertical skewing
            Object::Real(1.0), // d: vertical scaling
            Object::Real(info_x as f32), // e: horizontal position
            Object::Real(line_y as f32),  // f: vertical position
        ]));
        
        text_operations.push(Operation::new("Tj", vec![
            Object::string_literal(line.clone()),
        ])); // Show text
    }
    
    text_operations.push(Operation::new("ET", vec![])); // End text
    
    let content = Content { operations: text_operations };
    let content_data = content.encode()?;
    
    // Create a new content stream
    let stream = Stream::new(Dictionary::new(), content_data);
    let stream_id = doc.add_object(stream);
    
    // Get the page object and add stream to it
    if let Ok(page_obj) = doc.get_object_mut(page_id) {
        if let Ok(page_dict) = page_obj.as_dict_mut() {
            // Add to page's content array
            if let Ok(contents_obj) = page_dict.get_mut(b"Contents") {
                match contents_obj {
                    Object::Reference(_ref_id) => {
                        // For simplicity, replace the content reference with our new stream
                        *contents_obj = Object::Reference(stream_id);
                    },
                    Object::Array(ref mut contents_array) => {
                        contents_array.push(Object::Reference(stream_id));
                    },
                    _ => {
                        // Replace with new content stream
                        *contents_obj = Object::Reference(stream_id);
                    }
                }
            } else {
                // Add new Contents array
                page_dict.set(b"Contents", Object::Reference(stream_id));
            }
        }
    }
    
    Ok(())
}

// Helper function to extract filename from URL
fn extract_filename_from_url(url: &str) -> String {
    url.split('/').last().unwrap_or("file").to_string()
}

// Helper function to generate hash ID similar to frontend hashId function
fn hash_id(value: i64) -> String {
    let str_value = value.to_string();
    
    // Create 32-bit hash from value
    let mut hash: i32 = 0;
    for ch in str_value.chars() {
        hash = ((hash << 5).wrapping_sub(hash).wrapping_add(ch as i32)) | 0;
    }
    
    // Generate hex string (8 characters from 32-bit hash)
    let mut hex = String::new();
    for i in 0..8 {
        let h = ((hash >> (i * 4)) & 0xF) as u8;
        hex.push_str(&format!("{:X}", h));
    }
    
    // Repeat to get 32 characters: hex.len() = 8, we need 32, so repeat 4 times
    let hex32 = format!("{}{}{}{}", hex, hex, hex, hex);
    
    // Format as UUID (8-4-4-4-12 = 32 characters)
    format!(
        "{}-{}-{}-{}-{}",
        &hex32[0..8],
        &hex32[8..12],
        &hex32[12..16],
        &hex32[16..20],
        &hex32[20..32]
    )
}

// Calculate text height for signature info (matching SignatureRenderer.tsx logic)
fn calculate_signature_text_height(
    global_settings: &crate::database::models::DbGlobalSettings,
    submitter_id: Option<i64>,
    submitter_email: &str,
    reason: &str,
) -> f64 {
    let mut line_count = 0;
    
    if global_settings.add_signature_id_to_the_documents {
        if submitter_id.is_some() { line_count += 1; }
        if !submitter_email.is_empty() { line_count += 1; }
        line_count += 1; // date
    }
    
    if global_settings.require_signing_reason && !reason.is_empty() {
        line_count += 1;
    }
    
    // Match SignatureRenderer.tsx: (lineCount - 1) * 10 + 8 + 3
    if line_count > 0 {
        ((line_count - 1) as f64 * 10.0) + 8.0 + 3.0
    } else {
        0.0
    }
}

// Render signature ID information below the signature
fn render_signature_id_info(
    doc: &mut lopdf::Document,
    page_id: lopdf::ObjectId,
    submitter: &crate::database::models::DbSubmitter,
    signature_data: &serde_json::Value,
    x_pos: f64,
    pdf_y: f64,
    field_width: f64,
    field_height: f64,
    global_settings: &crate::database::models::DbGlobalSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    use lopdf::{Object, Stream, Dictionary};
    use lopdf::content::{Content, Operation};

    // Generate signature ID using hashId function (matching frontend)
    let signature_id = hash_id(submitter.id + 1);

    // Get reason from signature data
    let reason = signature_data.get("reason")
        .and_then(|r| r.as_str())
        .unwrap_or("");

    // Format the signature information
    let signer_email = submitter.email.clone();
    let signed_at = submitter.signed_at.unwrap_or(chrono::Utc::now());
    
    // Parse timezone from global settings or use default GMT+7
    let timezone_str = global_settings.timezone.as_deref().unwrap_or("Asia/Ho_Chi_Minh");
    
    // Map common timezone names to IANA identifiers (matching SignatureRenderer)
    let timezone_mapped = match timezone_str {
        "Midway Island" => "Pacific/Midway",
        "Hawaii" => "Pacific/Honolulu",
        "Alaska" => "America/Anchorage",
        "Pacific" => "America/Los_Angeles",
        "Mountain" => "America/Denver",
        "Central" => "America/Chicago",
        "Eastern" => "America/New_York",
        "Atlantic" => "America/Halifax",
        "Newfoundland" => "America/St_Johns",
        "London" => "Europe/London",
        "Berlin" => "Europe/Berlin",
        "Paris" => "Europe/Paris",
        "Rome" => "Europe/Rome",
        "Moscow" => "Europe/Moscow",
        "Tokyo" => "Asia/Tokyo",
        "Shanghai" => "Asia/Shanghai",
        "Hong Kong" => "Asia/Hong_Kong",
        "Singapore" => "Asia/Singapore",
        "Sydney" => "Australia/Sydney",
        "UTC" => "UTC",
        _ => timezone_str,
    };
    
    // Parse timezone offset (simplified approach for common timezones)
    let timezone_offset_hours = match timezone_mapped {
        "Asia/Ho_Chi_Minh" => 7,
        "Pacific/Midway" => -11,
        "Pacific/Honolulu" => -10,
        "America/Anchorage" => -9,
        "America/Los_Angeles" => -8,
        "America/Denver" => -7,
        "America/Chicago" => -6,
        "America/New_York" => -5,
        "America/Halifax" => -4,
        "Europe/London" => 0,
        "Europe/Berlin" | "Europe/Paris" | "Europe/Rome" => 1,
        "Europe/Moscow" => 3,
        "Asia/Tokyo" => 9,
        "Asia/Shanghai" | "Asia/Hong_Kong" | "Asia/Singapore" => 8,
        "Australia/Sydney" => 10,
        "UTC" => 0,
        _ => 7, // Default to GMT+7
    };
    
    let timezone_offset = chrono::FixedOffset::east_opt(timezone_offset_hours * 3600).unwrap();
    let signed_at_formatted = signed_at.with_timezone(&timezone_offset);
    
    // Format date according to locale (simplified)
    let locale = global_settings.locale.as_deref().unwrap_or("vi-VN");
    let date_str = if locale.starts_with("vi") {
        // Vietnamese format: DD/MM/YYYY, HH:MM:SS
        signed_at_formatted.format("%d/%m/%Y, %H:%M:%S").to_string()
    } else {
        // English/Default format: MM/DD/YYYY, HH:MM:SS
        signed_at_formatted.format("%m/%d/%Y, %H:%M:%S").to_string()
    };
    
    let mut signature_info_parts = Vec::new();
    
    // Always show reason first if require_signing_reason is enabled and reason exists
    if global_settings.require_signing_reason && !reason.is_empty() {
        signature_info_parts.push(format!("Reason: {}", reason));
    }
    
    // Show ID, email, and date if add_signature_id_to_the_documents is enabled
    if global_settings.add_signature_id_to_the_documents {
        signature_info_parts.push(format!("ID: {}", signature_id));
        signature_info_parts.push(signer_email.clone());
        signature_info_parts.push(date_str);
    }
    
    // If nothing to show, return early
    if signature_info_parts.is_empty() {
        return Ok(());
    }

    // Calculate text height dynamically (matching SignatureRenderer.tsx)
    let text_height = calculate_signature_text_height(
        global_settings,
        Some(submitter.id),
        &signer_email,
        reason
    );

    // Position the signature info at the BOTTOM of the field
    // Matching SignatureRenderer.tsx: text starts from bottom and goes up
    let info_x = x_pos + 5.0; // Match frontend padding of 5px
    let font_size = 8.0; // Match frontend font size
    let line_height = 10.0; // Match frontend line height
    
    // Text area is at the bottom: from pdf_y to (pdf_y + text_height)
    // Calculate actual text height needed
    let actual_text_height = (signature_info_parts.len() as f64 - 1.0) * line_height + font_size + 3.0;
    
    // Start rendering from the bottom of text area
    // First line should be at pdf_y + 3 (bottom padding), last line at the top
    let text_start_y = pdf_y + 3.0; // Bottom padding: 3px from the bottom
    
    // Create text content stream for signature info with multiple lines
    let mut text_operations = vec![
        Operation::new("BT", vec![]), // Begin text
        Operation::new("Tf", vec![
            Object::Name(b"Helvetica".to_vec()),
            Object::Real(font_size as f32),
        ]), // Set font
        Operation::new("rg", vec![
            Object::Real(0.0),
            Object::Real(0.0),
            Object::Real(0.0),
        ]), // Set text color to black
    ];
    
    // Render each line from bottom to top (matching SignatureRenderer.tsx)
    // SignatureRenderer draws: for (let i = textToShow.length - 1; i >= 0; i--
    let num_lines = signature_info_parts.len();
    for (idx, line) in signature_info_parts.iter().enumerate() {
        // Calculate Y position: start from bottom and go up
        // Line 0 (first in array) at bottom, line N-1 (last) at top
        let line_y = text_start_y + ((num_lines - 1 - idx) as f64 * line_height);
        
        // Use Tm (text matrix) to set absolute position for each line
        text_operations.push(Operation::new("Tm", vec![
            Object::Real(1.0), // a: horizontal scaling
            Object::Real(0.0), // b: horizontal skewing
            Object::Real(0.0), // c: vertical skewing
            Object::Real(1.0), // d: vertical scaling
            Object::Real(info_x as f32), // e: horizontal position
            Object::Real(line_y as f32),  // f: vertical position
        ]));
        
        text_operations.push(Operation::new("Tj", vec![
            Object::string_literal(line.clone()),
        ])); // Show text
    }
    
    text_operations.push(Operation::new("ET", vec![])); // End text
    
    let content = Content { operations: text_operations };
    let content_data = content.encode()?;
    
    // Create a new content stream
    let stream = Stream::new(Dictionary::new(), content_data);
    let stream_id = doc.add_object(stream);
    
    // Get the page object and add stream to it
    if let Ok(page_obj) = doc.get_object_mut(page_id) {
        if let Ok(page_dict) = page_obj.as_dict_mut() {
            // Add to page's content array
            if let Ok(contents_obj) = page_dict.get_mut(b"Contents") {
                match contents_obj {
                    Object::Reference(_ref_id) => {
                        // For simplicity, replace the content reference with our new stream
                        *contents_obj = Object::Reference(stream_id);
                    },
                    Object::Array(ref mut contents_array) => {
                        contents_array.push(Object::Reference(stream_id));
                    },
                    _ => {
                        // Replace with new content stream
                        *contents_obj = Object::Reference(stream_id);
                    }
                }
            } else {
                // Add new Contents array
                page_dict.set(b"Contents", Object::Reference(stream_id));
            }
        }
    }
    
    Ok(())
}

// Helper function to extract filename from URL
fn extract_filename_from_url(url: &str) -> String {
    url.split('/').last().unwrap_or("file").to_string()
}

// Helper function to generate hash ID similar to frontend hashId function
fn hash_id(value: i64) -> String {
    let str_value = value.to_string();
    
    // Create 32-bit hash from value
    let mut hash: i32 = 0;
    for ch in str_value.chars() {
        hash = ((hash << 5).wrapping_sub(hash).wrapping_add(ch as i32)) | 0;
    }
    
    // Generate hex string (8 characters from 32-bit hash)
    let mut hex = String::new();
    for i in 0..8 {
        let h = ((hash >> (i * 4)) & 0xF) as u8;
        hex.push_str(&format!("{:X}", h));
    }
    
    // Repeat to get 32 characters: hex.len() = 8, we need 32, so repeat 4 times
    let hex32 = format!("{}{}{}{}", hex, hex, hex, hex);
    
    // Format as UUID (8-4-4-4-12 = 32 characters)
    format!(
        "{}-{}-{}-{}-{}",
        &hex32[0..8],
        &hex32[8..12],
        &hex32[12..16],
        &hex32[16..20],
        &hex32[20..32]
    )
}

// Calculate text height for signature info (matching SignatureRenderer.tsx logic)
fn calculate_signature_text_height(
    global_settings: &crate::database::models::DbGlobalSettings,
    submitter_id: Option<i64>,
    submitter_email: &str,
    reason: &str,
) -> f64 {
    let mut line_count = 0;
    
    if global_settings.add_signature_id_to_the_documents {
        if submitter_id.is_some() { line_count += 1; }
        if !submitter_email.is_empty() { line_count += 1; }
        line_count += 1; // date
    }
    
    if global_settings.require_signing_reason && !reason.is_empty() {
        line_count += 1;
    }
    
    // Match SignatureRenderer.tsx: (lineCount - 1) * 10 + 8 + 3
    if line_count > 0 {
        ((line_count - 1) as f64 * 10.0) + 8.0 + 3.0
    } else {
        0.0
    }
}

// Render signature ID information below the signature
fn render_signature_id_info(
    doc: &mut lopdf::Document,
    page_id: lopdf::ObjectId,
    submitter: &crate::database::models::DbSubmitter,
    signature_data: &serde_json::Value,
    x_pos: f64,
    pdf_y: f64,
    field_width: f64,
    field_height: f64,
    global_settings: &crate::database::models::DbGlobalSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    use lopdf::{Object, Stream, Dictionary};
    use lopdf::content::{Content, Operation};

    // Generate signature ID using hashId function (matching frontend)
    let signature_id = hash_id(submitter.id + 1);

    // Get reason from signature data
    let reason = signature_data.get("reason")
        .and_then(|r| r.as_str())
        .unwrap_or("");

    // Format the signature information
    let signer_email = submitter.email.clone();
    let signed_at = submitter.signed_at.unwrap_or(chrono::Utc::now());
    
    // Parse timezone from global settings or use default GMT+7
    let timezone_str = global_settings.timezone.as_deref().unwrap_or("Asia/Ho_Chi_Minh");
    
    // Map common timezone names to IANA identifiers (matching SignatureRenderer)
    let timezone_mapped = match timezone_str {
        "Midway Island" => "Pacific/Midway",
        "Hawaii" => "Pacific/Honolulu",
        "Alaska" => "America/Anchorage",
        "Pacific" => "America/Los_Angeles",
        "Mountain" => "America/Denver",
        "Central" => "America/Chicago",
        "Eastern" => "America/New_York",
        "Atlantic" => "America/Halifax",
        "Newfoundland" => "America/St_Johns",
        "London" => "Europe/London",
        "Berlin" => "Europe/Berlin",
        "Paris" => "Europe/Paris",
        "Rome" => "Europe/Rome",
        "Moscow" => "Europe/Moscow",
        "Tokyo" => "Asia/Tokyo",
        "Shanghai" => "Asia/Shanghai",
        "Hong Kong" => "Asia/Hong_Kong",
        "Singapore" => "Asia/Singapore",
        "Sydney" => "Australia/Sydney",
        "UTC" => "UTC",
        _ => timezone_str,
    };
    
    // Parse timezone offset (simplified approach for common timezones)
    let timezone_offset_hours = match timezone_mapped {
        "Asia/Ho_Chi_Minh" => 7,
        "Pacific/Midway" => -11,
        "Pacific/Honolulu" => -10,
        "America/Anchorage" => -9,
        "America/Los_Angeles" => -8,
        "America/Denver" => -7,
        "America/Chicago" => -6,
        "America/New_York" => -5,
        "America/Halifax" => -4,
        "Europe/London" => 0,
        "Europe/Berlin" | "Europe/Paris" | "Europe/Rome" => 1,
        "Europe/Moscow" => 3,
        "Asia/Tokyo" => 9,
        "Asia/Shanghai" | "Asia/Hong_Kong" | "Asia/Singapore" => 8,
        "Australia/Sydney" => 10,
        "UTC" => 0,
        _ => 7, // Default to GMT+7
    };
    
    let timezone_offset = chrono::FixedOffset::east_opt(timezone_offset_hours * 3600).unwrap();
    let signed_at_formatted = signed_at.with_timezone(&timezone_offset);
    
    // Format date according to locale (simplified)
    let locale = global_settings.locale.as_deref().unwrap_or("vi-VN");
    let date_str = if locale.starts_with("vi") {
        // Vietnamese format: DD/MM/YYYY, HH:MM:SS
        signed_at_formatted.format("%d/%m/%Y, %H:%M:%S").to_string()
    } else {
        // English/Default format: MM/DD/YYYY, HH:MM:SS
        signed_at_formatted.format("%m/%d/%Y, %H:%M:%S").to_string()
    };
    
    let mut signature_info_parts = Vec::new();
    
    // Always show reason first if require_signing_reason is enabled and reason exists
    if global_settings.require_signing_reason && !reason.is_empty() {
        signature_info_parts.push(format!("Reason: {}", reason));
    }
    
    // Show ID, email, and date if add_signature_id_to_the_documents is enabled
    if global_settings.add_signature_id_to_the_documents {
        signature_info_parts.push(format!("ID: {}", signature_id));
        signature_info_parts.push(signer_email.clone());
        signature_info_parts.push(date_str);
    }
    
    // If nothing to show, return early
    if signature_info_parts.is_empty() {
        return Ok(());
    }

    // Calculate text height dynamically (matching SignatureRenderer.tsx)
    let text_height = calculate_signature_text_height(
        global_settings,
        Some(submitter.id),
        &signer_email,
        reason
    );

    // Position the signature info at the BOTTOM of the field
    // Matching SignatureRenderer.tsx: text starts from bottom and goes up
    let info_x = x_pos + 5.0; // Match frontend padding of 5px
    let font_size = 8.0; // Match frontend font size
    let line_height = 10.0; // Match frontend line height
    
    // Text area is at the bottom: from pdf_y to (pdf_y + text_height)
    // Calculate actual text height needed
    let actual_text_height = (signature_info_parts.len() as f64 - 1.0) * line_height + font_size + 3.0;
    
    // Start rendering from the bottom of text area
    // First line should be at pdf_y + 3 (bottom padding), last line at the top
    let text_start_y = pdf_y + 3.0; // Bottom padding: 3px from the bottom
    
    // Create text content stream for signature info with multiple lines
    let mut text_operations = vec![
        Operation::new("BT", vec![]), // Begin text
        Operation::new("Tf", vec![
            Object::Name(b"Helvetica".to_vec()),
            Object::Real(font_size as f32),
        ]), // Set font
        Operation::new("rg", vec![
            Object::Real(0.0),
            Object::Real(0.0),
            Object::Real(0.0),
        ]), // Set text color to black
    ];
    
    // Render each line from bottom to top (matching SignatureRenderer.tsx)
    // SignatureRenderer draws: for (let i = textToShow.length - 1; i >= 0; i--)
    let num_lines = signature_info_parts.len();
    for (idx, line) in signature_info_parts.iter().enumerate() {
        // Calculate Y position: start from bottom and go up
        // Line 0 (first in array) at bottom, line N-1 (last) at top
        let line_y = text_start_y + ((num_lines - 1 - idx) as f64 * line_height);
        
        // Use Tm (text matrix) to set absolute position for each line
        text_operations.push(Operation::new("Tm", vec![
            Object::Real(1.0), // a: horizontal scaling
            Object::Real(0.0), // b: horizontal skewing
            Object::Real(0.0), // c: vertical skewing
            Object::Real(1.0), // d: vertical scaling
            Object::Real(info_x as f32), // e: horizontal position
            Object::Real(line_y as f32),  // f: vertical position
        ]));
        
        text_operations.push(Operation::new("Tj", vec![
            Object::string_literal(line.clone()),
        ])); // Show text
    }
    
    text_operations.push(Operation::new("ET", vec![])); // End text
    
    let content = Content { operations: text_operations };
    let content_data = content.encode()?;
    
    // Create a new content stream
    let stream = Stream::new(Dictionary::new(), content_data);
    let stream_id = doc.add_object(stream);
    
    // Get the page object and add stream to it
    if let Ok(page_obj) = doc.get_object_mut(page_id) {
        if let Ok(page_dict) = page_obj.as_dict_mut() {
            // Add to page's content array
            if let Ok(contents_obj) = page_dict.get_mut(b"Contents") {
                match contents_obj {
                    Object::Reference(_ref_id) => {
                        // For simplicity, replace the content reference with our new stream
                        *contents_obj = Object::Reference(stream_id);
                    },
                    Object::Array(ref mut contents_array) => {
                        contents_array.push(Object::Reference(stream_id));
                    },
                    _ => {
                        // Replace with new content stream
                        *contents_obj = Object::Reference(stream_id);
                    }
                }
            } else {
                // Add new Contents array
                page_dict.set(b"Contents", Object::Reference(stream_id));
            }
        }
    }
    
    Ok(())
}

// Helper function to extract filename from URL
fn extract_filename_from_url(url: &str) -> String {
    url.split('/').last().unwrap_or("file").to_string()
}

// Helper function to generate hash ID similar to frontend hashId function
fn hash_id(value: i64) -> String {
    let str_value = value.to_string();
    
    // Create 32-bit hash from value
    let mut hash: i32 = 0;
    for ch in str_value.chars() {
        hash = ((hash << 5).wrapping_sub(hash).wrapping_add(ch as i32)) | 0;
    }
    
    // Generate hex string (8 characters from 32-bit hash)
    let mut hex = String::new();
    for i in 0..8 {
        let h = ((hash >> (i * 4)) & 0xF) as u8;
        hex.push_str(&format!("{:X}", h));
    }
    
    // Repeat to get 32 characters: hex.len() = 8, we need 32, so repeat 4 times
    let hex32 = format!("{}{}{}{}", hex, hex, hex, hex);
    
    // Format as UUID (8-4-4-4-12 = 32 characters)
    format!(
        "{}-{}-{}-{}-{}",
        &hex32[0..8],
        &hex32[8..12],
        &hex32[12..16],
        &hex32[16..20],
        &hex32[20..32]
    )
}

// Calculate text
import { useBasicSettings } from '@/hooks/useBasicSettings';
import React, { useRef, useEffect } from 'react';
import { hashId } from '../constants/reminderDurations';
interface SignatureRendererProps {
  data: string; // JSON string of point groups or typed text
  width?: number;
  height?: number;
  fieldType?: string;
  color?: string; // Color for signature/text
  additionalText?: string; // Additional text to display below the signature
  submitterId?: number;
  submitterEmail?: string;
  reason?: string; // Signing reason to display/ Signing reason to display
}

const SignatureRenderer: React.FC<SignatureRendererProps> = ({
  data,
  width = 200,
  height = 100,
  fieldType,
  color = '#000000',
  additionalText,
  submitterId,
  submitterEmail,
  reason
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const { globalSettings } = useBasicSettings();
  useEffect(() => {
    // Map common timezone names to IANA identifiers
    const timeZoneMap: Record<string, string> = {
      "Midway Island": "Pacific/Midway",
      "Hawaii": "Pacific/Honolulu",
      "Alaska": "America/Anchorage",
      "Pacific": "America/Los_Angeles",
      "Mountain": "America/Denver",
      "Central": "America/Chicago",
      "Eastern": "America/New_York",
      "Atlantic": "America/Halifax",
      "Newfoundland": "America/St_Johns",
      "London": "Europe/London",
      "Berlin": "Europe/Berlin",
      "Paris": "Europe/Paris",
      "Rome": "Europe/Rome",
      "Moscow": "Europe/Moscow",
      "Tokyo": "Asia/Tokyo",
      "Shanghai": "Asia/Shanghai",
      "Hong Kong": "Asia/Hong_Kong",
      "Singapore": "Asia/Singapore",
      "Sydney": "Australia/Sydney",
      "UTC": "UTC"
    };

    let timeZone = 'Asia/Ho_Chi_Minh';
    const configuredTimeZone = globalSettings?.timezone;
    if (configuredTimeZone) {
      const mappedTimeZone = timeZoneMap[configuredTimeZone] || configuredTimeZone;
      try {
        new Intl.DateTimeFormat('en', { timeZone: mappedTimeZone });
        timeZone = mappedTimeZone;
      } catch {
        // Invalid time zone, use default
      }
    }
    const locale = globalSettings?.locale || 'vi-VN';
    const dateOptions: Intl.DateTimeFormatOptions = {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      timeZone
    };

    const canvas = canvasRef.current;
    if (!canvas) {
      console.error('Canvas ref not available');
      return;
    }

    const ctx = canvas.getContext('2d');
    if (!ctx) {
      console.error('Cannot get canvas context');
      return;
    }

    // Enable high-quality rendering for images, but crisp rendering for vectors
    ctx.imageSmoothingEnabled = true;
    ctx.imageSmoothingQuality = 'high';

    // Clear canvas
    ctx.clearRect(0, 0, width, height);

    // Check if data is an image URL (including blob URLs)
    if (data && (data.startsWith('http') || data.startsWith('/') || data.startsWith('blob:'))) {
      // Render image
      const img = new Image();
      // Only set crossOrigin for non-blob URLs
      if (!data.startsWith('blob:')) {
        img.crossOrigin = 'anonymous';
      }
      img.onload = () => {
        // Calculate text height dynamically
        let textHeight = 0;
        if (globalSettings?.add_signature_id_to_the_documents || additionalText || (globalSettings?.require_signing_reason && reason)) {
          // Estimate text height: 12px per line + 6px padding
          let lineCount = 0;
          if (globalSettings?.add_signature_id_to_the_documents) {
            lineCount += (submitterId ? 1 : 0) + (submitterEmail ? 1 : 0) + 1; // date
          } else if (additionalText) {
            lineCount += 1;
          }
          if (globalSettings?.require_signing_reason && reason) {
            lineCount += 1;
          }
          textHeight = lineCount > 0 ? (lineCount - 1) * 6 + 8 + 2 + 10 : 0; // More precise: (lines-1)*lineHeight + fontSize + padding + gap
        }

        // Calculate scale to fit image in canvas, leaving space for text if needed
        const scale = Math.min(width / img.width, (height - textHeight) / img.height);
        const scaledWidth = img.width * scale;
        const scaledHeight = img.height * scale;

        // Center the image in the available space
        const offsetX = (width - scaledWidth) / 2;
        const offsetY = ((height - textHeight) - scaledHeight) / 2 + 5; // Moved up by 5 pixels

        // Clear canvas again before drawing
        ctx.clearRect(0, 0, width, height);

        // Enable high-quality image rendering
        ctx.imageSmoothingEnabled = true;
        ctx.imageSmoothingQuality = 'high';

        ctx.drawImage(img, offsetX, offsetY, scaledWidth, scaledHeight);



        // Render additional text below the image if enabled
        let textToShow: string[] = [];
        if (globalSettings?.add_signature_id_to_the_documents) {
          if (submitterId) textToShow.push(`ID: ${hashId(submitterId + 1)}`);
          if (submitterEmail) textToShow.push(submitterEmail);
          textToShow.push(new Date().toLocaleString(locale, dateOptions));
        } else if (additionalText) {
          textToShow = [additionalText];
        }

        // Always show reason if require_signing_reason is enabled and reason exists
        if (globalSettings?.require_signing_reason && reason) {
          if (globalSettings?.add_signature_id_to_the_documents) {
            // Show both reason and ID/email/date
            textToShow = [`Reason: ${reason}`, `ID: ${hashId(submitterId + 1)}`, submitterEmail, new Date().toLocaleString(locale, dateOptions)].filter(Boolean);
          } else {
            // Show only reason
            textToShow = [`Reason: ${reason}`];
          }
        }

        if (textToShow.length > 0) {
          ctx.fillStyle = color;
          ctx.font = '8px sans-serif';
          ctx.textAlign = 'left';
          ctx.textBaseline = 'bottom';

          // Calculate line height
          const lineHeight = 10;
          let y = height - 3;

          // Draw lines from bottom to top
          for (let i = textToShow.length - 1; i >= 0; i--) {
            ctx.fillText(textToShow[i], 5, y);
            y -= lineHeight;
          }


        }
      };
      img.onerror = (error) => {
        console.error('Image failed to load:', data, error);
        // Fallback to text if image fails to load
        ctx.clearRect(0, 0, width, height);
        ctx.fillStyle = color;
        ctx.font = '12px sans-serif';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText('Image failed to load', width / 2, height / 2);

        // Render additional text below the fallback text if enabled
        const textToShow = additionalText || (globalSettings?.add_signature_id_to_the_documents ?
          new Date().toLocaleString(locale, dateOptions) : null);

        if (textToShow) {
          ctx.fillStyle = color;
          ctx.font = '8px sans-serif';
          ctx.textAlign = 'center';
          ctx.textBaseline = 'bottom';
          ctx.fillText(textToShow, width / 2, height - 3);
        }
      };
      img.src = data;
      return;
    }

    try {
      const pointGroups = JSON.parse(data);

      if (!pointGroups || pointGroups.length === 0) {
        throw new Error('Empty data');
      }

      // Disable image smoothing for crisp vector lines
      ctx.imageSmoothingEnabled = false;

      // It's vector data, render as before
      // Find bounds of signature to scale it properly
      let minX = Infinity, minY = Infinity;
      let maxX = -Infinity, maxY = -Infinity;

      pointGroups.forEach((group: any[]) => {
        group.forEach((point: any) => {
          minX = Math.min(minX, point.x);
          minY = Math.min(minY, point.y);
          maxX = Math.max(maxX, point.x);
          maxY = Math.max(maxY, point.y);
        });
      });

      const signatureWidth = maxX - minX;
      const signatureHeight = maxY - minY;

      // Calculate text height dynamically
      let textHeight = 0;
      if (globalSettings?.add_signature_id_to_the_documents || additionalText || (globalSettings?.require_signing_reason && reason)) {
        // Estimate text height: 12px per line + 6px padding
        let lineCount = 0;
        if (globalSettings?.add_signature_id_to_the_documents) {
          lineCount += (submitterId ? 1 : 0) + (submitterEmail ? 1 : 0) + 1; // date
        } else if (additionalText) {
          lineCount += 1;
        }
        if (globalSettings?.require_signing_reason && reason) {
          lineCount += 1;
        }
        textHeight = lineCount > 0 ? (lineCount - 1) * 6 + 8 + 2 + 10 : 0; // More precise: (lines-1)*lineHeight + fontSize + padding + gap
      }

      // Calculate scale to fit signature in canvas with minimal padding, leaving space for text if needed
      const padding = 2;
      const scaleX = (width - padding * 2) / signatureWidth;
      const scaleY = ((height - textHeight) - padding * 2) / signatureHeight;
      const scale = Math.min(scaleX, scaleY); // Use minimum scale to preserve aspect ratio

      // Calculate offset to center signature in available space, but move to top
      const offsetX = (width - signatureWidth * scale) / 2 - minX * scale;
      const offsetY = padding - minY * scale; // Move to top with minimal padding

      // Draw signature with natural line width similar to original drawing
      ctx.strokeStyle = color;
      ctx.lineWidth = 2.5; // Fixed width that matches typical signature pen thickness
      ctx.lineCap = 'round';
      ctx.lineJoin = 'round';
      ctx.globalAlpha = 1.0; // Ensure full opacity
      ctx.miterLimit = 10; // Prevent sharp corners from being too pointy

      pointGroups.forEach((group: any[]) => {
        if (group.length === 0) return;

        ctx.beginPath();
        group.forEach((point: any, index: number) => {
          const x = point.x * scale + offsetX;
          const y = point.y * scale + offsetY;

          if (index === 0) {
            ctx.moveTo(x, y);
          } else {
            ctx.lineTo(x, y);
          }
        });
        ctx.stroke();
      });



      // Re-enable image smoothing for text and images
      ctx.imageSmoothingEnabled = true;

      // Render additional text below the signature if enabled
      let textToShow: string[] = [];
      if (globalSettings?.add_signature_id_to_the_documents) {
        if (submitterId) textToShow.push(`ID: ${hashId(submitterId + 1)}`);
        if (submitterEmail) textToShow.push(submitterEmail);
        textToShow.push(new Date().toLocaleString(locale, dateOptions));
      } else if (additionalText) {
        textToShow = [additionalText];
      }

      // Always show reason if require_signing_reason is enabled and reason exists
      if (globalSettings?.require_signing_reason && reason) {
        if (globalSettings?.add_signature_id_to_the_documents) {
          // Show both reason and ID/email/date
          textToShow = [`Reason: ${reason}`, `ID: ${hashId(submitterId + 1)}`, submitterEmail, new Date().toLocaleString(locale, dateOptions)].filter(Boolean);
        } else {
          // Show only reason
          textToShow = [`Reason: ${reason}`];
        }
      }

      if (textToShow.length > 0) {
        ctx.fillStyle = color;
        ctx.font = '8px sans-serif';
        ctx.textAlign = 'left';
        ctx.textBaseline = 'bottom';

        // Calculate line height
        const lineHeight = 8;
        let y = height - 2;

        // Draw lines from bottom to top
        for (let i = textToShow.length - 1; i >= 0; i--) {
          ctx.fillText(textToShow[i], 5, y);
          y -= lineHeight;
        }

      }
    } catch (e) {
      if (fieldType === 'initials') {
        // Calculate text height dynamically
        let textHeight = 0;
        if (globalSettings?.add_signature_id_to_the_documents || additionalText || (globalSettings?.require_signing_reason && reason)) {
          // Estimate text height: 12px per line + 6px padding
          let lineCount = 0;
          if (globalSettings?.add_signature_id_to_the_documents) {
            lineCount += (submitterId ? 1 : 0) + (submitterEmail ? 1 : 0) + 1; // date
          } else if (additionalText) {
            lineCount += 1;
          }
          if (globalSettings?.require_signing_reason && reason) {
            lineCount += 1;
          }
          textHeight = lineCount > 0 ? (lineCount - 1) * 6 + 8 + 2 + 10 : 0; // More precise: (lines-1)*lineHeight + fontSize + padding + gap
        }

        const scale = 3;
        const scaledWidth = width * scale;
        const scaledHeight = height * scale;

        // Set canvas to scaled size for better quality
        canvas.width = scaledWidth;
        canvas.height = scaledHeight;
        ctx.scale(scale, scale); // Apply scale

        const fontFamily = 'Helvetica';
        const fontStyle = 'italic';
        const fontWeight = 'normal';

        // Start with a large font size
        let fontSize = Math.max((height - textHeight) * 0.6, 10);
        fontSize = Math.min(fontSize, 18); // Match PDF font size calculation
        ctx.font = `${fontStyle} ${fontWeight} ${fontSize}px ${fontFamily}`;

        // Measure actual text bounds
        let metrics = ctx.measureText(data);
        let actualHeight = metrics.actualBoundingBoxAscent + metrics.actualBoundingBoxDescent;

        ctx.fillStyle = color;
        ctx.textAlign = 'center';
        ctx.textBaseline = 'alphabetic';
        ctx.globalAlpha = 1.0; // Ensure full opacity

        // Clear and draw
        ctx.clearRect(0, 0, width, height);

        // Calculate Y to center text vertically in available space, then push up for bottom
        const centerY = ((height - textHeight) - actualHeight) / 2 + metrics.actualBoundingBoxAscent - ((height - textHeight) * 0.01) + 5; // Moved up by 5 pixels

        ctx.fillText(data, width / 2, centerY);

        // Reset transform after scaling
        ctx.setTransform(1, 0, 0, 1, 0, 0);

        // Render additional text below the initials if enabled
        let textToShow: string[] = [];
        if (globalSettings?.add_signature_id_to_the_documents) {
          if (submitterId) textToShow.push(`ID: ${hashId(submitterId + 1)}`);
          if (submitterEmail) textToShow.push(submitterEmail);
          textToShow.push(new Date().toLocaleString(locale, dateOptions));
        } else if (additionalText) {
          textToShow = [additionalText];
        }

        // Always show reason if require_signing_reason is enabled and reason exists
        if (globalSettings?.require_signing_reason && reason) {
          if (globalSettings?.add_signature_id_to_the_documents) {
            // Show both reason and ID/email/date
            textToShow = [`Reason: ${reason}`, `ID: ${hashId(submitterId + 1)}`, submitterEmail, new Date().toLocaleString(locale, dateOptions)].filter(Boolean);
          } else {
            // Show only reason
            textToShow = [`Reason: ${reason}`];
          }
        }

        if (textToShow.length > 0) {
          ctx.fillStyle = color;
          ctx.font = '8px sans-serif';
          ctx.textAlign = 'left';
          ctx.textBaseline = 'bottom';

          // Calculate line height
          const lineHeight = 8;
          let y = height - 2;

          // Draw lines from bottom to top
          for (let i = textToShow.length - 1; i >= 0; i--) {
            ctx.fillText(textToShow[i], 5, y);
            y -= lineHeight;
          }

        }
      } else {
        // Calculate text height dynamically
        let textHeight = 0;
        if (globalSettings?.add_signature_id_to_the_documents || additionalText || (globalSettings?.require_signing_reason && reason)) {
          // Estimate text height: 12px per line + 6px padding
          let lineCount = 0;
          if (globalSettings?.add_signature_id_to_the_documents) {
            lineCount += (submitterId ? 1 : 0) + (submitterEmail ? 1 : 0) + 1; // date
          } else if (additionalText) {
            lineCount += 1;
          }
          if (globalSettings?.require_signing_reason && reason) {
            lineCount += 1;
          }
          textHeight = lineCount > 0 ? (lineCount - 1) * 6 + 8 + 2 + 10 : 0; // More precise: (lines-1)*lineHeight + fontSize + padding + gap
        }

        // Default text rendering for signatures
        ctx.fillStyle = color;
        let fontSize = 12; // Match PDF font size
        ctx.font = `${fontSize}px sans-serif`;

        // Check if text fits in width and scale down if needed
        let metrics = ctx.measureText(data || '');
        let textWidth = metrics.width;
        if (textWidth > width * 0.95) { // Allow 95% of width
          fontSize = (width * 0.95 / textWidth) * fontSize;
          ctx.font = `${fontSize}px sans-serif`;
          metrics = ctx.measureText(data || '');
        }

        // ctx.textAlign = 'center';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.globalAlpha = 1.0; // Ensure full opacity
        ctx.fillText(data || '', width / 2, (height - textHeight) / 2 + 5); // Moved up by 5 pixels

        // Render additional text below the default text if enabled
        let textToShow: string[] = [];
        if (globalSettings?.add_signature_id_to_the_documents) {
          if (submitterId) textToShow.push(`ID: ${hashId(submitterId + 1)}`);
          if (submitterEmail) textToShow.push(submitterEmail);
          textToShow.push(new Date().toLocaleString(locale, dateOptions));
        } else if (additionalText) {
          textToShow = [additionalText];
        }

        // Always show reason if require_signing_reason is enabled and reason exists
        if (globalSettings?.require_signing_reason && reason) {
          if (globalSettings?.add_signature_id_to_the_documents) {
            // Show both reason and ID/email/date
            textToShow = [`Reason: ${reason}`, `ID: ${hashId(submitterId + 1)}`, submitterEmail, new Date().toLocaleString(locale, dateOptions)].filter(Boolean);
          } else {
            // Show only reason
            textToShow = [`Reason: ${reason}`];
          }
        }

        if (textToShow.length > 0) {
          ctx.fillStyle = color;
          ctx.font = '8px sans-serif';
          ctx.textAlign = 'left';
          ctx.textBaseline = 'bottom';

          // Calculate line height
          const lineHeight = 8;
          let y = height - 2;

          // Draw lines from bottom to top
          for (let i = textToShow.length - 1; i >= 0; i--) {
            ctx.fillText(textToShow[i], 5, y);
            y -= lineHeight;
          }

        }
      }
    }
  }, [data, width, height, fieldType, color, additionalText, submitterId, submitterEmail, reason, globalSettings]);

  return (
    <canvas
      ref={canvasRef}
      width={width}
      height={height}
      style={{
        width: '100%',
        height: '100%',
        maxWidth: '100%',
        maxHeight: '100%',
        imageRendering: 'auto',
        // border: 'none',
        // outline: 'none'
      }}
    />
  );
};

export default SignatureRenderer;
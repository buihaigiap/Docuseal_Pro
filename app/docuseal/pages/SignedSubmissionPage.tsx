import React, { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { SubmissionSignaturesResponse } from '../types';
import PdfViewer from '../components/PdfViewer';
import upstashService from '../ConfigApi/upstashService';
import toast from 'react-hot-toast';
import { PDFDocument, rgb, StandardFonts } from 'pdf-lib';
import { hashId } from '../constants/reminderDurations';
import { useBasicSettings } from '../hooks/useBasicSettings';

// Helper function to render vector signature to canvas and convert to image
const renderSignatureToImage = (signatureData: string, width: number, height: number, options?: {
  submitterId?: number;
  submitterEmail?: string;
  reason?: string;
  additionalText?: string;
  globalSettings?: any;
}): Promise<string> => {
  return new Promise((resolve, reject) => {
    try {
      console.log('Starting renderSignatureToImage:', { width, height, dataLength: signatureData.length });
      
      // Use provided dimensions (already clamped by caller)
      const canvasWidth = Math.round(width);
      const canvasHeight = Math.round(height);
      
      // Safety check: ensure dimensions are reasonable
      if (canvasWidth > 2000 || canvasHeight > 2000 || canvasWidth < 50 || canvasHeight < 50) {
        console.warn('Canvas dimensions out of range, using defaults:', canvasWidth, canvasHeight);
        reject(new Error(`Invalid canvas dimensions: ${canvasWidth}x${canvasHeight}`));
        return;
      }
      
      const canvas = document.createElement('canvas');
      canvas.width = canvasWidth;
      canvas.height = canvasHeight;
      const ctx = canvas.getContext('2d');
      
      if (!ctx) {
        reject(new Error('Cannot get canvas context'));
        return;
      }

      // Parse signature data
      const pointGroups = JSON.parse(signatureData);
      
      console.log('Parsed point groups:', pointGroups.length, 'groups');
      
      if (!pointGroups || pointGroups.length === 0) {
        reject(new Error('Empty signature data'));
        return;
      }

      // Clear canvas WITHOUT background (transparent)
      ctx.clearRect(0, 0, canvasWidth, canvasHeight);

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

      console.log('Signature bounds:', { minX, minY, maxX, maxY });

      const signatureWidth = maxX - minX;
      const signatureHeight = maxY - minY;
      
      if (signatureWidth <= 0 || signatureHeight <= 0) {
        reject(new Error('Invalid signature dimensions'));
        return;
      }
      
      console.log('=== PDF DOWNLOAD SIGNATURE RENDER ===');
      console.log('Canvas dimensions:', { canvasWidth, canvasHeight });
      console.log('Signature dimensions:', { signatureWidth, signatureHeight });
      
      // Calculate text height dynamically (giống SignatureRenderer)
      let textHeight = 0;
      if (options?.globalSettings?.add_signature_id_to_the_documents || (options?.globalSettings?.require_signing_reason && options?.reason)) {
        // Estimate text height: 12px per line + 6px padding
        let lineCount = 0;
        if (options?.globalSettings?.add_signature_id_to_the_documents) {
          lineCount += (options?.submitterId ? 1 : 0) + (options?.submitterEmail ? 1 : 0) + 1; // date
        }
        if (options?.globalSettings?.require_signing_reason && options?.reason) {
          lineCount += 1;
        }
        textHeight = lineCount > 0 ? (lineCount - 1) * 10 + 8 + 3 : 0; // More precise: (lines-1)*lineHeight + fontSize + padding
      }
      
      console.log('Text height calculation:', { textHeight, globalSettings_add_signature_id: options?.globalSettings?.add_signature_id_to_the_documents, require_signing_reason: options?.globalSettings?.require_signing_reason, reason: options?.reason });
      
      // Calculate scale to fit signature in canvas with minimal padding, giống web viewer
      const padding = 5;
      const scaleX = (canvasWidth - padding * 2) / signatureWidth;
      const scaleY = ((canvasHeight - textHeight) - padding * 2) / signatureHeight;
      const scale = Math.min(scaleX, scaleY); // Use minimum scale to preserve aspect ratio

      console.log('Scale calculation:', { scaleX, scaleY, scale, padding });
      
      // Calculate offset to center signature
      const offsetX = (canvasWidth - signatureWidth * scale) / 2 - minX * scale;
      const offsetY = ((canvasHeight - textHeight) - signatureHeight * scale) / 2 - minY * scale;

      console.log('Positioning:', { offsetX, offsetY });
      console.log('====================================');

      // Draw signature with natural line width similar to web viewer
      ctx.strokeStyle = '#000000';
      ctx.lineWidth = 2.5; // Match web viewer thickness
      ctx.lineCap = 'round';
      ctx.lineJoin = 'round';
      ctx.globalAlpha = 1.0; // Ensure full opacity
      ctx.miterLimit = 10; // Prevent sharp corners

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

      // Re-enable image smoothing for text
      ctx.imageSmoothingEnabled = true;

      // Render additional text below the signature if enabled (giống SignatureRenderer)
      const { submitterId, submitterEmail, reason, additionalText, globalSettings } = options || {};
      
      let textToShow: string[] = [];
      if (globalSettings?.add_signature_id_to_the_documents) {
        if (submitterId) textToShow.push(`ID: ${hashId(submitterId + 1)}`);
        if (submitterEmail) textToShow.push(submitterEmail);
        textToShow.push(new Date().toLocaleString('vi-VN', {
          year: 'numeric', month: '2-digit', day: '2-digit',
          hour: '2-digit', minute: '2-digit', second: '2-digit',
          timeZone: 'Asia/Ho_Chi_Minh'
        }));
      } else if (additionalText) {
        textToShow = [additionalText];
      }

      // Always show reason if require_signing_reason is enabled and reason exists
      if (globalSettings?.require_signing_reason && reason) {
        if (globalSettings?.add_signature_id_to_the_documents) {
          // Show both reason and ID/email/date
          textToShow = [`Reason: ${reason}`, `ID: ${hashId(submitterId + 1)}`, submitterEmail, new Date().toLocaleString('vi-VN', {
            year: 'numeric', month: '2-digit', day: '2-digit',
            hour: '2-digit', minute: '2-digit', second: '2-digit',
            timeZone: 'Asia/Ho_Chi_Minh'
          })].filter(Boolean);
        } else {
          // Show only reason
          textToShow = [`Reason: ${reason}`];
        }
      }

      if (textToShow.length > 0) {
        ctx.fillStyle = '#000000';
        ctx.font = '8px sans-serif';
        ctx.textAlign = 'left';
        ctx.textBaseline = 'bottom';
        
        // Calculate line height
        const lineHeight = 10;
        let y = canvasHeight - 3;
        
        // Draw lines from bottom to top
        for (let i = textToShow.length - 1; i >= 0; i--) {
          ctx.fillText(textToShow[i], 5, y);
          y -= lineHeight;
        }
      }

      console.log('Drawing complete, converting to data URL');

      // Convert canvas to data URL
      const imageDataUrl = canvas.toDataURL('image/png');
      
      // Verify the data URL is valid
      if (!imageDataUrl || imageDataUrl.length < 100 || !imageDataUrl.startsWith('data:image/png')) {
        reject(new Error('Failed to create valid PNG data URL'));
        return;
      }
      
      console.log('✅ Image data URL created, length:', imageDataUrl.length);
      resolve(imageDataUrl);
    } catch (error) {
      console.error('❌ Error in renderSignatureToImage:', error);
      reject(error);
    }
  });
};

const handleDownload = async (data: SubmissionSignaturesResponse, token: string, submitterInfo: { id: number; email: string } | null, globalSettings: any) => {
  // Fetch PDF file từ server với binary response
  const API_BASE_URL = (import.meta as any).env?.VITE_API_BASE_URL || '';
  const fullUrl = `${API_BASE_URL}/api/files/${data.template_info.document.url}`;
  const response = await fetch(fullUrl, {
    headers: {
      'Authorization': localStorage.getItem('token') ? `Bearer ${localStorage.getItem('token')}` : ''
    }
  });
  
  if (!response.ok) {
    throw new Error(`Failed to fetch PDF: ${response.statusText}`);
  }
  
  const pdfBytes = await response.arrayBuffer();

  // Load PDF với pdf-lib
  const pdfDoc = await PDFDocument.load(pdfBytes);
  const pages = pdfDoc.getPages();
  const font = await pdfDoc.embedFont(StandardFonts.Helvetica);

  // Lặp qua tất cả chữ ký và render lên PDF
  for (const signature of data.bulk_signatures) {
    const field = signature.field_info;
    const signatureValue = signature.signature_value;

    console.log('Processing field:', {
      name: field.name,
      type: field.field_type,
      hasValue: !!signatureValue,
      valuePreview: signatureValue?.substring(0, 50),
      position: field.position
    });

    if (!signatureValue || !field.position) continue;

    const pageIndex = field.position.page - 1; // Convert 1-based to 0-based
    if (pageIndex < 0 || pageIndex >= pages.length) continue;

    const page = pages[pageIndex];
    const { width: pageWidth, height: pageHeight } = page.getSize();
    
    console.log('Page dimensions:', { pageWidth, pageHeight, pageIndex });
    console.log('Field position (raw):', field.position);
    
    // Normalize position giống như web viewer (sử dụng default PDF dimensions 600x800)
    const normalizePosition = (position: any) => {
      if (!position || typeof position.x !== 'number') return position;
      
      // Check if position is in pixels (values > 1) or already in decimal (0-1)
      if (position.x > 1 || position.y > 1 || position.width > 1 || position.height > 1) {
        // Position is in pixels, convert to relative (0-1) using DEFAULT PDF dimensions như web viewer
        const DEFAULT_PDF_WIDTH = 600;
        const DEFAULT_PDF_HEIGHT = 800;
        return {
          ...position,
          x: position.x / DEFAULT_PDF_WIDTH,
          y: position.y / DEFAULT_PDF_HEIGHT,
          width: position.width / DEFAULT_PDF_WIDTH,
          height: position.height / DEFAULT_PDF_HEIGHT
        };
      }
      // Already in relative format
      return position;
    };
    
    const normalizedPos = normalizePosition(field.position);
    console.log('Field position (normalized):', normalizedPos);
    
    // DÙNG CÔNG THỨC GIỐNG FRONTEND (PdfViewer.tsx)
    // Position trong database là pixel values, normalize về relative (0-1) dùng default 600x800 như web viewer
    const x = Math.max(0, Math.min(1, normalizedPos.x)) * pageWidth;
    const y = Math.max(0, Math.min(1, normalizedPos.y)) * pageHeight;
    const fieldWidth = Math.max(0, Math.min(1, normalizedPos.width)) * pageWidth;
    const fieldHeight = Math.max(0, Math.min(1, normalizedPos.height)) * pageHeight;

    console.log('Calculated positions (clamped):', { x, y, fieldWidth, fieldHeight });

    // DEBUG: Compare with web viewer positioning
    console.log('=== POSITION DEBUG ===');
    console.log('Raw position from DB (pixels):', field.position);
    console.log('Normalized position (using 600x800 default):', normalizedPos);
    console.log('Web-style CSS positioning would be:');
    console.log(`  left: ${normalizedPos.x * 100}%`);
    console.log(`  top: ${normalizedPos.y * 100}%`);
    console.log(`  width: ${normalizedPos.width * 100}%`);
    console.log(`  height: ${normalizedPos.height * 100}%`);
    console.log('PDF positioning (actual page dimensions):');
    console.log(`  fieldWidth: ${fieldWidth}, fieldHeight: ${fieldHeight}`);
    console.log('======================');

    // PDF coordinates: bottom-left origin, nhưng ta cần convert từ top-left
    const pdfX = Math.max(0, Math.min(pageWidth - fieldWidth, x));
    const pdfY = Math.max(0, pageHeight - y - fieldHeight);

    console.log(`  pdfX: ${pdfX}, pdfY: ${pdfY}`);

    // Render based on field type
    if (field.field_type === 'text' || field.field_type === 'date' || field.field_type === 'number') {
      // Render text
      const fontSize = Math.min(fieldHeight * 0.6, 12);
      page.drawText(signatureValue, {
        x: pdfX,
        y: pdfY + fieldHeight * 0.3, // Center vertically
        size: fontSize,
        font: font,
        color: rgb(0, 0, 0),
      });
    } else if (field.field_type === 'signature' || field.field_type === 'initials') {
    console.log('=== PDF DOWNLOAD FIELD POSITIONING ===');
    console.log('Field ID:', field.id);
    console.log('Field type:', field.field_type);
    console.log('Raw position from DB:', field.position);
    console.log('Normalized position (using default 600x800 like web viewer):', normalizedPos);
    console.log('Actual PDF page dimensions:', { pageWidth, pageHeight });
    console.log('Calculated field dimensions:', { fieldWidth, fieldHeight });
    console.log('PDF coordinates:', { pdfX, pdfY });
    console.log('====================================');

      // Xử lý chữ ký (có thể là image hoặc drawn signature)
      if (signatureValue.startsWith('data:image/')) {
        // Chữ ký dạng image - embed vào PDF
        try {
          const imageBytes = await fetch(signatureValue).then(res => res.arrayBuffer());
          let image;
          if (signatureValue.includes('png')) {
            image = await pdfDoc.embedPng(imageBytes);
          } else {
            image = await pdfDoc.embedJpg(imageBytes);
          }
          
          // Scale image to fit field
          const imgDims = image.scale(1);
          const scale = Math.min(fieldWidth / imgDims.width, fieldHeight / imgDims.height);
          
          page.drawImage(image, {
            x: pdfX,
            y: pdfY,
            width: imgDims.width * scale,
            height: imgDims.height * scale,
          });
        } catch (err) {
          console.error('Error embedding signature image:', err);
        }
      } else if (signatureValue.startsWith('[') || signatureValue.startsWith('{')) {
        // Vector signature data - render to canvas then embed as image
        // Use EXACT field dimensions như web viewer, không clamp
        const canvasWidth = fieldWidth;
        const canvasHeight = fieldHeight;
        
        console.log('Rendering vector signature, canvas:', canvasWidth, 'x', canvasHeight, 'PDF field:', fieldWidth, 'x', fieldHeight);
        try {
          // Render signature to canvas and get image data
          const signatureImageUrl = await renderSignatureToImage(
            signatureValue,
            canvasWidth,
            canvasHeight,
            {
              submitterId: submitterInfo?.id,
              submitterEmail: submitterInfo?.email,
              reason: signature.reason,
              globalSettings
            }
          );
          
          console.log('Vector signature rendered to image, size:', signatureImageUrl.length);
          
          // Embed the rendered signature image at the exact field dimensions
          const imageBytes = await fetch(signatureImageUrl).then(res => res.arrayBuffer());
          const image = await pdfDoc.embedPng(imageBytes);
          
          console.log('Embedding signature at:', { x: pdfX, y: pdfY, width: fieldWidth, height: fieldHeight });
          
          page.drawImage(image, {
            x: pdfX,
            y: pdfY,
            width: fieldWidth,
            height: fieldHeight,
          });
          
          console.log('✅ Vector signature embedded successfully');
        } catch (err) {
          console.error('Error rendering vector signature:', err);
          // Fallback to text placeholder
          const fontSize = Math.min(fieldHeight * 0.6, 12);
          page.drawText('[Signature]', {
            x: pdfX,
            y: pdfY + fieldHeight * 0.3,
            size: fontSize,
            font: font,
            color: rgb(0, 0, 0),
          });
        }
      } else {
        // Plain text signature
        const fontSize = Math.min(fieldHeight * 0.6, 12);
        page.drawText(signatureValue, {
          x: pdfX,
          y: pdfY + fieldHeight * 0.3,
          size: fontSize,
          font: font,
          color: rgb(0, 0, 0),
        });
      }
    } else if (field.field_type === 'checkbox') {
      // Render checkbox
      if (signatureValue === 'true') {
        // Draw checkmark
        const checkSize = Math.min(fieldWidth, fieldHeight) * 0.8;
        page.drawText('✓', {
          x: pdfX + (fieldWidth - checkSize) / 2,
          y: pdfY + (fieldHeight - checkSize) / 2,
          size: checkSize,
          font: font,
          color: rgb(0, 0, 0),
        });
      }
    } else if (field.field_type === 'image') {
      // Handle uploaded images
      if (signatureValue.startsWith('http') || signatureValue.startsWith('blob:') || signatureValue.startsWith('data:image/')) {
        try {
          const imageBytes = await fetch(signatureValue).then(res => res.arrayBuffer());
          let image;
          if (signatureValue.includes('png')) {
            image = await pdfDoc.embedPng(imageBytes);
          } else {
            image = await pdfDoc.embedJpg(imageBytes);
          }
          
          const imgDims = image.scale(1);
          const scale = Math.min(fieldWidth / imgDims.width, fieldHeight / imgDims.height);
          
          page.drawImage(image, {
            x: pdfX,
            y: pdfY,
            width: imgDims.width * scale,
            height: imgDims.height * scale,
          });
        } catch (err) {
          console.error('Error embedding image:', err);
        }
      }
    }
  }

  // Save và download PDF
  const pdfBytesModified = await pdfDoc.save();
  const blob = new Blob([pdfBytesModified as any], { type: 'application/pdf' });
  const link = document.createElement('a');
  link.href = URL.createObjectURL(blob);
  link.download = `signed_${data.template_info.name}.pdf`;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(link.href);
};

const SignedSubmissionPage = () => {
  const { token } = useParams<{ token: string }>();
  const navigate = useNavigate();
  const [data, setData] = useState<SubmissionSignaturesResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [downloading, setDownloading] = useState(false);
  const [isMobile, setIsMobile] = useState(window.innerWidth < 768);
  useEffect(() => {
    const handleResize = () => setIsMobile(window.innerWidth < 768);
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);
  const [submitterInfo, setSubmitterInfo] = useState<{ id: number; email: string } | null>(null);
  const { globalSettings } = useBasicSettings();
  useEffect(() => {
    const fetchData = async () => {
      try {
        // Fetch both signatures and fields data in parallel
        const [signaturesResult, fieldsResult] = await Promise.all([
          upstashService.getSubmissionSignatures(token),
          upstashService.getSubmissionFields(token)
        ]);

        console.log('Signatures Result:', signaturesResult);
        console.log('Fields Result:', fieldsResult);

        if (signaturesResult.success) {
          setData(signaturesResult.data);
        } else {
          setError(signaturesResult.message || 'Failed to fetch signatures data');
        }

        if (fieldsResult.success && fieldsResult.data.information) {
          setSubmitterInfo({
            id: fieldsResult.data.information.id,
            email: fieldsResult.data.information.email
          });
        }
      } catch (err) {
        console.error('Fetch error:', err);
        setError('An error occurred while fetching data');
      } finally {
        setLoading(false);
      }
    };

    if (token) {
      fetchData();
    }
  }, [token]);

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-500 mx-auto mb-4"></div>
          <p>Loading submission...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
        <div className="text-center">
          <p className="text-red-500 mb-4">{error}</p>
          <button onClick={() => navigate(-1)} className="px-4 py-2 bg-indigo-600 rounded-md hover:bg-indigo-700">
            Go Back
          </button>
        </div>
      </div>
    );
  }

  const handleDownloadClick = async () => {
    if (!data || !token) return;
    
    setDownloading(true);
    try {
      await handleDownload(data, token, submitterInfo, globalSettings);
      toast.success('PDF downloaded successfully!');
    } catch (err: any) {
      console.error('Download error:', err);
      toast.error(`Failed to download PDF: ${err.message || 'Unknown error'}`);
    } finally {
      setDownloading(false);
    }
  };

  if (!data) return null;

  return (
    <div className="min-h-screen bg-gray-900 text-white p-4">
      {/* Header with Download Button */}
      <div className="max-w-7xl mx-auto mb-6">
        <div className="flex items-center justify-between">
          <button 
            onClick={() => navigate(-1)} 
            className="px-4 py-2 bg-gray-700 rounded-md hover:bg-gray-600 transition-colors flex items-center gap-2"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 19l-7-7m0 0l7-7m-7 7h18" />
            </svg>
            Back
          </button>
          
          <div className="flex items-center gap-4">
            <h1 className="text-xl font-semibold">{data.template_info.name}</h1>
            <button
              onClick={handleDownloadClick}
              disabled={downloading}
              className={`px-4 py-2 rounded-md transition-colors flex items-center gap-2 ${
                downloading 
                  ? 'bg-gray-600 cursor-not-allowed' 
                  : 'bg-indigo-600 hover:bg-indigo-700'
              }`}
            >
              {downloading ? (
                <>
                  <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                  Downloading...
                </>
              ) : (
                <>
                  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                  </svg>
                  Download PDF
                </>
              )}
            </button>
          </div>
        </div>
      </div>

      {/* PDF Viewer */}
      <div className={`max-w-7xl mx-auto ${isMobile ? 'relative' : 'grid grid-cols-1 lg:grid-cols-3 gap-6'}`}>
        <div className={`${isMobile ? 'w-full' : 'lg:col-span-2'}`}>
          <PdfViewer
            filePath={data.template_info.document.url}
            fields={data?.bulk_signatures?.map(sig => ({ ...sig.field_info, signature_value: sig.signature_value, reason: sig.reason }))}
            submitterId={submitterInfo?.id}
            submitterEmail={submitterInfo?.email}
            // scale={1.5}
          />
        </div>
      </div>
    </div>
  );
};

export default SignedSubmissionPage;
import React, { useState, useEffect, useCallback, useRef } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';
import { TemplateFullInfo, Submitter, NewSubmitter, NewTemplateField, Template, SubmissionSignaturesResponse } from '../types';
import Toast from '../components/Toast';
import InviteModal from '../components/InviteModal';
import upstashService from '../ConfigApi/upstashService';
import { PDFDocument, rgb } from 'pdf-lib';
import { Box, Button, CircularProgress, Typography, Paper, Grid, Alert, useMediaQuery } from '@mui/material';
import { useTheme } from '@mui/material/styles';
import { Trash2 , Copy } from 'lucide-react';
const TemplateDetailPage = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { token } = useAuth();
  const [templateInfo, setTemplateInfo] = useState<TemplateFullInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [showInviteModal, setShowInviteModal] = useState(false);
  const [partnerEmails, setPartnerEmails] = useState<Record<string, string>>({});
  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('sm'));
  const [snackbar, setSnackbar] = useState<{open: boolean, message: string, severity: 'success' | 'error' | 'info' | 'warning'}>({open: false, message: '', severity: 'info'});
  const fetchTemplateInfo = useCallback(async () => {
    setLoading(true);
    try {
      const data = await upstashService.getTemplateFullInfo(parseInt(id));
      if (data.success) {
        const templateData = data.data;
        const template: Template = {
          id: templateData.template.id,
          name: templateData.template.name,
          file_url: templateData.template.documents?.[0]?.url || '',
          documents: templateData.template.documents,
          created_at: templateData.template.created_at,
          user_id: templateData.template.user_id,
          slug: templateData.template.slug,
          updated_at: templateData.template.updated_at,
          fields: templateData.template.template_fields.map((f: any) => ({
            id: f.id,
            name: f.name,
            field_type: f.field_type,
            required: f.required,
            position: f.position,
            display_order: f.display_order,
            options: f.options,
            partner: f.partner
          }))
        };
        
        // Extract signatures with their parties
        const signatures = templateData.signatures || [];
        
        // Flatten submitters for backward compatibility
        const submitters: Submitter[] = [];
        signatures.forEach((signature: any) => {
          if (signature.parties) {
            signature.parties.forEach((party: any) => {
              submitters.push({
                id: party.id,
                name: party.name,
                email: party.email,
                status: party.status,
                token: party.token,
                created_at: party.created_at,
                signed_at: party.signed_at,
                template_id: party.template_id,
                user_id: party.user_id,
                updated_at: party.updated_at
              });
            });
          }
        });
        
        setTemplateInfo({
          template,
          submitters,
          total_submitters: submitters.length,
          signatures
        });
        const partners = [...new Set(template.fields?.map(f => f.partner).filter(Boolean) || [])];
        setPartnerEmails(Object.fromEntries(partners.map(p => [p, ''])));
      } else {
        console.error('❌ API call failed:', data.message || 'Unknown error');
        setError(data.message || 'Failed to fetch template details.');
      }
    } catch (err) {
      console.error('💥 API fetch failed with exception:', err, 'isMobile:', isMobile);
      // Mock data for development
      const mockTemplate: Template = {
        id: parseInt(id!),
        name: 'Sample Template',
        file_url: 'https://example.com/sample.pdf',
        documents: [{ url: 'https://example.com/sample.pdf' }],
        created_at: new Date().toISOString(),
        user_id: 1,
        slug: 'sample-template',
        updated_at: new Date().toISOString(),
        fields: [
          {
            id: 1,
            name: 'Full Name',
            field_type: 'text',
            required: true,
            position: { x: 100, y: 200, width: 200, height: 30, page: 0 },
            display_order: 1
          },
          {
            id: 2,
            name: 'Signature',
            field_type: 'signature',
            required: true,
            position: { x: 100, y: 250, width: 200, height: 50, page: 0 },
            display_order: 2
          }
        ]
      };
      setTemplateInfo({
        template: mockTemplate,
        submitters: [],
        total_submitters: 0
      });
    } finally {
      setLoading(false);
    }
  }, [id, token]);

  useEffect(() => {
    fetchTemplateInfo();
  }, [fetchTemplateInfo]);

  const handleCreateSubmission = async (e: React.FormEvent) => {
    e.preventDefault();
    const submitters = Object.entries(partnerEmails).map(([partner, email]) => ({ name: partner, email }));
    if (submitters.some(s => !s.email)) {
      setSnackbar({open: true, message: 'Please fill in all email addresses.', severity: 'warning'});
      return;
    }

    try {
      const data = await upstashService.createSubmission({ template_id: parseInt(id!), submitters });
      if (data.success) {
        setSnackbar({open: true, message: 'Submission created successfully! Signers have been invited.', severity: 'success'});
        fetchTemplateInfo();
        setPartnerEmails(Object.fromEntries(Object.keys(partnerEmails).map(p => [p, ''])));
        setShowInviteModal(false);
      } else {
        setSnackbar({open: true, message: data.error || 'Error creating submission', severity: 'error'});
      }
    } catch (err) {
      setSnackbar({open: true, message: 'An unexpected error occurred.', severity: 'error'});
    } finally {
    }
  };

  const handleViewSubmission = (submissionToken: string) => {
    navigate(`/signed-submission/${submissionToken}`);
  };
  const downloadSignedPDF = async (submitter: Submitter) => {
    try {
      // Fetch submission signatures
      const signaturesData = await upstashService.getSubmissionSignatures(submitter.token);
      
      if (!signaturesData.bulk_signatures || signaturesData.bulk_signatures.length === 0) {
        throw new Error('No signatures found for this submission');
      }

      // Fetch the original PDF
      const pdfUrl = signaturesData.template_info.document.url;
      if (!pdfUrl) {
        throw new Error('PDF URL not found');
      }

      const pdfResponse = await upstashService.previewFile(pdfUrl);
      const pdfBytes = pdfResponse.data;

      // Load the PDF
      const pdfDoc = await PDFDocument.load(pdfBytes);
      const pages = pdfDoc.getPages();

      // Draw signatures on the PDF
      for (const signature of signaturesData.bulk_signatures) {
        const field = signature.field_info;

        if (signature.signature_value && field.position) {
          const page = pages[field.position.page];
          const { width, height } = page.getSize();

          // Calculate position (PDF coordinates are from bottom-left)
          const x = field.position.x;
          const y = height - field.position.y - field.position.height;

          // Draw signature
          if (signature.signature_value.startsWith('data:image/')) {
            // It's an image signature
            try {
              const imageBytes = await fetch(signature.signature_value).then(res => res.arrayBuffer());
              const image = signature.signature_value.includes('png') 
                ? await pdfDoc.embedPng(imageBytes)
                : await pdfDoc.embedJpg(imageBytes);

              page.drawImage(image, {
                x,
                y,
                width: field.position.width,
                height: field.position.height,
              });
            } catch (err) {
              console.error('Failed to embed image:', err);
            }
          } else if (signature.signature_value.startsWith('[') || signature.signature_value.startsWith('{')) {
            // It's vector data (JSON) - convert to image first
            try {
              const canvas = document.createElement('canvas');
              canvas.width = field.position.width * 2; // 2x for better quality
              canvas.height = field.position.height * 2;
              const ctx = canvas.getContext('2d');
              
              if (ctx) {
                // Don't fill background - keep it transparent
                
                // Parse and draw the signature data
                const pointGroups = JSON.parse(signature.signature_value);
                ctx.strokeStyle = 'black';
                ctx.lineWidth = 2;
                ctx.lineCap = 'round';
                ctx.lineJoin = 'round';
                
                // Find bounds to scale properly
                let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
                for (const group of pointGroups) {
                  for (const point of group) {
                    minX = Math.min(minX, point.x);
                    minY = Math.min(minY, point.y);
                    maxX = Math.max(maxX, point.x);
                    maxY = Math.max(maxY, point.y);
                  }
                }
                
                const scaleX = canvas.width / (maxX - minX + 20);
                const scaleY = canvas.height / (maxY - minY + 20);
                const scale = Math.min(scaleX, scaleY);
                const offsetX = (canvas.width - (maxX - minX) * scale) / 2;
                const offsetY = (canvas.height - (maxY - minY) * scale) / 2;
                
                // Draw each stroke
                for (const group of pointGroups) {
                  if (group.length === 0) continue;
                  ctx.beginPath();
                  const firstPoint = group[0];
                  ctx.moveTo((firstPoint.x - minX) * scale + offsetX, (firstPoint.y - minY) * scale + offsetY);
                  
                  for (let i = 1; i < group.length; i++) {
                    const point = group[i];
                    ctx.lineTo((point.x - minX) * scale + offsetX, (point.y - minY) * scale + offsetY);
                  }
                  ctx.stroke();
                }
                
                // Convert canvas to PNG
                const dataUrl = canvas.toDataURL('image/png');
                const imageBytes = await fetch(dataUrl).then(res => res.arrayBuffer());
                const image = await pdfDoc.embedPng(imageBytes);
                
                page.drawImage(image, {
                  x,
                  y,
                  width: field.position.width,
                  height: field.position.height,
                });
              }
            } catch (err) {
              console.error('Failed to render vector signature:', err);
            }
          } else {
            // It's text
            const text = signature.signature_value.startsWith('data:')
              ? atob(signature.signature_value.split(',')[1])
              : signature.signature_value;

            page.drawText(text, {
              x,
              y: y + field.position.height / 2,
              size: 12,
              color: rgb(0, 0, 0),
            });
          }
        }
      }

      // Save and download
      const pdfBytesModified = await pdfDoc.save();
      const blob = new Blob([pdfBytesModified as any], { type: 'application/pdf' });
      const link = document.createElement('a');
      link.href = URL.createObjectURL(blob);
      link.download = `signed_${signaturesData.template_info.name}_${submitter.email}.pdf`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(link.href);

    } catch (err: any) {
      console.error('Download error:', err);
      setSnackbar({open: true, message: `Failed to download PDF: ${err.message}`, severity: 'error'});
    }
  };

  const handleDeleteSubmitter = async (submitterId: number) => {
    try {
      const data = await upstashService.deleteSubmitter(submitterId);
      if (data.success) {
        setSnackbar({open: true, message: 'Submitter deleted successfully!', severity: 'success'});
        fetchTemplateInfo(); // Refresh the template info to update the UI
      } else {
        setSnackbar({open: true, message: data.message || 'Error deleting submitter', severity: 'error'});
      }
    } catch (err) {
      console.error('Delete error:', err);
      setSnackbar({open: true, message: 'An unexpected error occurred while deleting the submitter.', severity: 'error'});
    }
  };

  const handleClone = async () => {
    try {
      const data = await upstashService.cloneTemplate(id);
      if (data.success) {
        setSnackbar({open: true, message: 'Template cloned successfully!', severity: 'success'});
        navigate(`/templates/${data.data.id}`);
      } else {
        setSnackbar({open: true, message: data.error || 'Error cloning template', severity: 'error'});
      }
    } catch (err) {
      setSnackbar({open: true, message: 'An unexpected error occurred while cloning.', severity: 'error'});
    }
  };

  const handleDelete = async () => {
    if (!confirm(`Are you sure you want to delete the template "${templateInfo?.template.name}"? This action cannot be undone.`)) {
      return;
    }

    try {
      const data = await upstashService.deleteTemplate(parseInt(id!));
      if (data.success) {
        setSnackbar({open: true, message: 'Template deleted successfully!', severity: 'success'});
        navigate('/'); // Navigate back to dashboard after deletion
      } else {
        setSnackbar({open: true, message: data.error || 'Error deleting template', severity: 'error'});
      }
    } catch (err) {
      setSnackbar({open: true, message: 'An unexpected error occurred while deleting the template.', severity: 'error'});
    }
  };

  if (loading) return <div className="text-center p-8">Loading template...</div>;
  if (error) return <div className="text-center text-red-500 p-8 bg-gray-800 rounded-lg">{error}</div>;
  if (!templateInfo) return <div className="text-center p-8">Template not found.</div>;

  return (
    <Box
      sx={{
         maxWidth: { xs: '100%', lg: 1400 }, 
         mx: 'auto', position: 'relative', zIndex: 1, px: { xs: 2, sm: 3, md: 4 }
       }}
    >
      <Box >
        <Grid container spacing={2} alignItems="center" justifyContent="space-between">
          <div >
            <Typography
              variant={isMobile ? "h5" : "h3"} component="h1" gutterBottom>
            Templates: {templateInfo?.template.name}
            </Typography>
          </div>
          <Grid >
            <Box sx={{ display: 'flex', gap: 1, flexDirection: isMobile ? 'column' : 'row' }}>
              <Button 
              sx={
                { color: 'grey.300' }
              }
              variant="outlined" onClick={() => setShowInviteModal(true)}>
                Invite to Sign
              </Button>
              <Button 
                sx={
                  { color: 'grey.300' }
                }
                startIcon={<Copy size={16} />}
                variant="outlined"
                onClick={handleClone}>
                  Clone
              </Button>
              <Button 
              sx={
                { color: 'red', borderColor: 'red' }
              }
              startIcon={<Trash2 size={16} />}
              variant="outlined"
               onClick={handleDelete}>
                Delete
              </Button>

              <Button variant="contained" onClick={() => navigate(`/templates/${id}/editor`)}>
                Edit Template
              </Button>
            </Box>
          </Grid>
        </Grid>
      </Box>

      {loading && <CircularProgress />}
      {error && <Alert severity="error">{error}</Alert>}

      {templateInfo && (
        <div className="mt-6">
          {templateInfo.signatures && templateInfo.signatures.length > 0 ? (
            <div className="space-y-6">
              <div className="flex justify-between items-center">
                <h2 className="text-2xl font-semibold">Signing Status</h2>
                <button onClick={() => setShowInviteModal(true)} className="px-4 py-2 font-semibold text-white bg-indigo-600 rounded-md hover:bg-indigo-700">
                  Add Recipients
                </button>
              </div>
              <div className="space-y-4">
                {templateInfo.signatures.map((signature: any, signatureIndex: number) => (
                  <div key={signatureIndex} className="bg-white/5 border border-white/10 rounded-lg p-4 border">
                    <div className="flex items-center justify-between mb-3 text-gray-500">
                      <h3 className="text-lg font-medium text-white">
                        {signature.type === 'bulk' ? 'Bulk Signature' : 'Single Signature'} 
                        <span className="text-sm  ml-2">
                          ({signature.parties.length} parties)
                        </span>
                      </h3>
                      <span className={`px-3 py-1 text-xs font-bold rounded-full uppercase ${
                        signature.overall_status === 'completed' 
                          ? 'bg-green-100 text-green-800' 
                          : 'bg-yellow-100 text-yellow-800'
                      }`}>
                        {signature.overall_status}
                      </span>
                    </div>
                    {signature.type === 'bulk' ? (
                      <div className="flex justify-between items-center rounded-lg  shadow-sm ">
                        <div className="space-y-2 flex-1">
                          {signature.parties.map((party: any) => (
                            <div key={party.id} className="flex items-center justify-between py-2 px-3  rounded">
                              <div className="flex items-center gap-3">
                                <span className={`px-3 py-1 text-xs font-bold rounded-full uppercase flex-shrink-0 ${
                                  party.status === 'signed' || party.status === 'completed' 
                                    ? 'bg-cyan-400 text-gray-800' 
                                    : 'bg-cyan-400 text-gray-800'
                                }`}>
                                  {party.status === 'signed' || party.status === 'completed' ? 'SIGNED' : 'SENT'}
                                </span>
                                <div>
                                  <h3 className="font-medium text-white text-sm">{party.name}</h3>
                                  <div className="text-xs">{party.email}</div>
                                </div>
                              </div>                             
                            </div>
                          ))}
                        </div>
                          <div className="flex items-center gap-2">
                            {signature.overall_status === 'completed' && (
                              <button 
                                onClick={(e) => {
                                  e.stopPropagation();
                                  downloadSignedPDF(signature.parties[0]);
                                }}
                                className="px-3 py-1.5 text-sm font-semibold text-gray-800
                                 border-2 border-gray-800 rounded-full 
                                 hover:bg-gray-800 hover:text-white transition-colors 
                                 flex items-center gap-1
                                 "
                              >
                                <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                                </svg>
                                DOWNLOAD
                              </button>
                            )}
                            <button 
                              onClick={() => handleViewSubmission(signature.parties[0].token)}

                              className="px-3 py-1.5 text-sm font-semibold 
                                   border-gray-500 , border
                                 rounded-full hover:bg-gray-800
                                  hover:text-white transition-colors"
                            >
                              VIEW
                            </button>
                            <button 
                              onClick={async (e) => {
                                e.stopPropagation();
                                if (confirm(`Are you sure you want to delete this bulk signature with ${signature.parties.length} parties?`)) {
                                  try {
                                    // Delete all parties in the bulk signature
                                    const deletePromises = signature.parties.map(party => 
                                      upstashService.deleteSubmitter(party.id)
                                    );
                                    await Promise.all(deletePromises);
                                    setSnackbar({open: true, message: 'Bulk signature deleted successfully!', severity: 'success'});
                                    fetchTemplateInfo();
                                  } catch (err) {
                                    console.error('Bulk delete error:', err);
                                    setSnackbar({open: true, message: 'An error occurred while deleting the bulk signature.', severity: 'error'});
                                  }
                                }
                              }}
                              className="p-1.5 text-gray-600 hover:text-red-600 transition-colors"
                            >
                                 <Trash2  color='red'/>
                            </button>
                        </div>
                     
                      </div>
                    ) : (
                      <div className="space-y-2">
                        {signature.parties.map((party: any) => (
                          <div key={party.id} className="rounded-lg p-3 flex items-center justify-between shadow-sm">
                            <div className="flex items-center gap-3">
                              <span className={`px-3 py-1 text-xs font-bold rounded-full uppercase ${
                                party.status === 'signed' || party.status === 'completed' 
                                  ? 'bg-cyan-400 text-gray-800' 
                                  : 'bg-cyan-400 text-gray-800'
                              }`}>
                                {party.status === 'signed' || party.status === 'completed' ? 'SIGNED' : 'SENT'}
                              </span>
                              <div>
                                <div className="font-medium">{party.name}</div>
                                <div className="text-xs  ">{party.email}</div>
                              </div>
                            </div>
                            <div className="flex items-center gap-2">
                              {party.status === 'pending' && (
                                <button 
                                  onClick={(e) => { 
                                    e.stopPropagation(); 
                                    window.open(`${window.location.origin}/templates/${party.token}/edit`, '_blank');
                                  }} 
                                  className="
                                  px-3 py-1.5 text-sm font-semibold 
                                  border border-gray-500
                                   rounded-full hover:bg-gray-800 
                                   hover:text-white transition-colors
                                    flex items-center gap-1"
                                >
                                  <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                                  </svg>
                                  SIGN NOW
                                </button>
                              )}
                              {(party.status === 'signed' || party.status === 'completed') && (
                                <button 
                                  onClick={(e) => {
                                    e.stopPropagation();
                                    downloadSignedPDF(party);
                                  }}
                                   className="
                                  px-3 py-1.5 text-sm font-semibold 
                                  border border-gray-500
                                   rounded-full hover:bg-gray-800 
                                   hover:text-white transition-colors
                                    flex items-center gap-1"
                                >
                                  <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                                  </svg>
                                  DOWNLOAD
                                </button>
                              )}
                              <button 
                                onClick={() => handleViewSubmission(party.token)}
                                className="px-3 py-1.5 text-sm font-semibold 
                                  border border-gray-500
                                 rounded-full hover:bg-gray-800
                                  hover:text-white transition-colors"
                              >
                                VIEW
                              </button>
                              {signature.type === 'single' && (
                                <button 
                                  onClick={(e) => {
                                    e.stopPropagation();
                                    if (confirm(`Are you sure you want to delete the submission for ${party.email}?`)) {
                                      handleDeleteSubmitter(party.id);
                                    }
                                  }}
                                  className="p-1.5 text-gray-600 hover:text-red-600 transition-colors"
                                >
                                      <Trash2  color='red'/>
                                </button>
                              )}
                            </div>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </div>
          ) : (
            <div className="text-center py-12">
              <h2 className="text-2xl font-semibold mb-4">No Signers Yet</h2>
              <p className="text-gray-400 mb-6">Send this document to recipients for signing.</p>
              <button onClick={() => setShowInviteModal(true)} className="px-6 py-3 font-semibold text-white bg-indigo-600 rounded-md hover:bg-indigo-700">
                Send to Recipients
              </button>
            </div>
          )}
        </div>
      )}

      {/* Invite Modal */}
      <InviteModal
        open={showInviteModal}
        onClose={() => setShowInviteModal(false)}
        partnerEmails={partnerEmails}
        onPartnerEmailsChange={setPartnerEmails}
        onSubmit={handleCreateSubmission}
      />   
        <Toast
          open={snackbar.open}
          message={snackbar.message}
          severity={snackbar.severity}
          onClose={() => setSnackbar({...snackbar, open: false})}
        />
    </Box>
  );
};

export default TemplateDetailPage;
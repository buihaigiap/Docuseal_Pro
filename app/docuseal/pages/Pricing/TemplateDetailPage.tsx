import React, { useState, useEffect, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useAuth } from '../../contexts/AuthContext';
import { TemplateFullInfo, Submitter, Template } from '../../types';
import InviteModal from '../../components/InviteModal';
import upstashService from '../../ConfigApi/upstashService';
import { Box, Button, CircularProgress, Typography, Grid, Alert, useMediaQuery } from '@mui/material';
import { useTheme } from '@mui/material/styles';
import { Trash2 , Copy } from 'lucide-react';
import toast from 'react-hot-toast';
import { canTemplate, useRoleAccess } from '../../hooks/useRoleAccess';
import SigningStatus from './TemplateDetailComponents/SigningStatus';
import { downloadSignedPDF as downloadSignedPDFFunc } from './TemplateDetailComponents/downloadUtils';
const TemplateDetailPage = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { token  , user} = useAuth();
  const [templateInfo, setTemplateInfo] = useState<TemplateFullInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [showInviteModal, setShowInviteModal] = useState(false);
  const [partnerEmails, setPartnerEmails] = useState<Record<string, string>>({});
  const [submitting, setSubmitting] = useState(false);
  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('sm'));
  const checkRole = canTemplate(templateInfo?.template);
  const hasAccess = useRoleAccess(['agent']);
  
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
          })),
          user_name: ''
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
        fields: [],
        user_name: ''
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
    console.log('useEffect triggered, calling fetchTemplateInfo');
    fetchTemplateInfo();
  }, [fetchTemplateInfo]);

  const handleCreateSubmission = async (e: React.FormEvent) => {
    e.preventDefault();
    const submitters = Object.entries(partnerEmails).map(([partner, email]) => ({ name: partner, email }));
    if (submitters.some(s => !s.email)) {
      toast.error('Please fill in all email addresses.');
      return;
    }
    setSubmitting(true);
    try {
      const data = await upstashService.createSubmission({ template_id: parseInt(id!), submitters });
      console.log('Create submission response:', data);
      if (data.success) {
        toast.success('Submission created successfully! Signers have been invited.');
        fetchTemplateInfo();
        setPartnerEmails(Object.fromEntries(Object.keys(partnerEmails).map(p => [p, ''])));
        setShowInviteModal(false);
      }
      if(user?.free_usage_count === 10) {
        toast.error('Free usage limit reached');
        navigate(`/pricing`);
        return;
      }
    } catch (err) {
        toast.error(err.response);
    } finally {
      setSubmitting(false);
    }
  };

  const handleViewSubmission = (submissionToken: string) => {
    navigate(`/signed-submission/${submissionToken}`);
  };
const downloadSignedPDF = async (submitter: Submitter, pdfUrl?: string) => {
    await downloadSignedPDFFunc(submitter, pdfUrl);
};
  const handleDeleteSubmitter = async (submitterId: number) => {
    try {
      const data = await upstashService.deleteSubmitter(submitterId);
      if (data.success) {
        toast.success('Submitter deleted successfully!');
        fetchTemplateInfo(); // Refresh the template info to update the UI
      } else {
        toast.error(data.message || 'Error deleting submitter');
      }
    } catch (err) {
      console.error('Delete error:', err);
      toast.error('An unexpected error occurred while deleting the submitter.');
    }
  };

  const handleClone = async () => {
    try {
      const data = await upstashService.cloneTemplate(id);
      if (data.success) {
        toast.success('Template cloned successfully!');
        navigate(`/templates/${data.data.id}`);
      } else {
        toast.error(data.error || 'Error cloning template');
      }
    } catch (err) {
      toast.error('An unexpected error occurred while cloning.');
    }
  };

  const handleDelete = async () => {
    if (!confirm(`Are you sure you want to delete the template "${templateInfo?.template.name}"? This action cannot be undone.`)) {
      return;
    }

    try {
      const data = await upstashService.deleteTemplate(parseInt(id!));
      if (data.success) {
        toast.success('Template deleted successfully!');
        navigate('/');
      } else {
        toast.error(data.error || 'Error deleting template');
      }
    } catch (err) {
      toast.error('An unexpected error occurred while deleting the template.');
    }
  };


  return (
    <Box>
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
              {!hasAccess && (
                <>
                  {canTemplate(templateInfo?.template) && (
                    <Button 
                      sx={
                        { color: 'grey.300' }
                      }
                      startIcon={<Copy size={16} />}
                      variant="outlined"
                      onClick={handleClone}>
                        Clone
                    </Button>
                  )}
                </>
              )}
               {!hasAccess && (
                 <>
                  {canTemplate(templateInfo?.template) && (
                  <Button 
                    sx={
                      { color: 'red', borderColor: 'red' }
                    }
                      startIcon={<Trash2 size={16} />}
                      variant="outlined"
                      onClick={handleDelete}>
                        Delete
                    </Button>
                  )}
                 </>
               )}

              <Button variant="contained" onClick={() => navigate(`/templates/${id}/editor`)}>
                 {!checkRole || hasAccess ? 'View Template' : 'Edit Template'}
              </Button>
            </Box>
          </Grid>
        </Grid>
      </Box>

      {loading && <CircularProgress />}
      {error && <Alert severity="error">{error}</Alert>}

      {templateInfo && (
        <SigningStatus
          templateInfo={templateInfo}
          handleViewSubmission={handleViewSubmission}
          downloadSignedPDF={downloadSignedPDF}
          handleDeleteSubmitter={handleDeleteSubmitter}
          fetchTemplateInfo={fetchTemplateInfo}
          setShowInviteModal={setShowInviteModal}
        />
      )}

      <InviteModal
        open={showInviteModal}
        onClose={() => setShowInviteModal(false)}
        partnerEmails={partnerEmails}
        onPartnerEmailsChange={setPartnerEmails}
        onSubmit={handleCreateSubmission}
        loading={submitting}
      />
    </Box>
  );
};

export default TemplateDetailPage;
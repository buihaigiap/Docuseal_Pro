import { useState, useEffect } from 'react';
import {
    Box,Button,TextField,Dialog,
    DialogTitle,DialogContent,DialogActions,FormControlLabel,Tabs,Tab,Paper,FormControl,InputLabel,Select,MenuItem,
} from '@mui/material';
import {Switch} from '@mui/material';
import CreateTemplateButton from '@/components/CreateTemplateButton';

interface EmailTemplate {
  id: number;
  user_id: number;
  template_type: string;
  subject: string;
  body: string;
  body_format: string;
  is_default: boolean;
  attach_documents?: boolean;
  attach_audit_log?: boolean;
  created_at: string;
  updated_at: string;
}

interface EmailTemplateDialogProps {
  open: boolean;
  onClose: () => void;
  editingTemplate: EmailTemplate | null;
  onSave: (data: any) => void;
  loading: boolean;
}

export default function EmailTemplateDialog({
  open,
  onClose,
  editingTemplate,
  onSave,
  loading
}: EmailTemplateDialogProps) {
  const [formData, setFormData] = useState({
    template_type: 'invitation',
    subject: '',
    body: '',
    body_format: 'text',
    is_default: false,
    attach_documents: false,
    attach_audit_log: false,
  });
  const [activeTab, setActiveTab] = useState(0);
  const [bodyText, setBodyText] = useState('');
  const [bodyHtml, setBodyHtml] = useState('');

  useEffect(() => {
    if (editingTemplate) {
      setFormData({
        template_type: editingTemplate.template_type,
        subject: editingTemplate.subject,
        body: editingTemplate.body,
        body_format: editingTemplate.body_format,
        is_default: editingTemplate.is_default,
        attach_documents: editingTemplate.attach_documents || false,
        attach_audit_log: editingTemplate.attach_audit_log || false,
      });
      setBodyText(editingTemplate.body_format === 'text' ? editingTemplate.body : '');
      setBodyHtml(editingTemplate.body_format === 'html' ? editingTemplate.body : '');
    } else {
      setFormData({
        template_type: 'invitation',
        subject: '',
        body: '',
        body_format: 'text',
        is_default: false,
        attach_documents: false,
        attach_audit_log: false,
      });
      setBodyText('');
      setBodyHtml('');
    }
    setActiveTab(formData.body_format === 'html' ? 1 : 0);
  }, [editingTemplate, open]);

  useEffect(() => {
    setActiveTab(formData.body_format === 'html' ? 1 : 0);
  }, [formData.body_format]);

  useEffect(() => {
    setFormData(prev => ({ ...prev, body: activeTab === 0 ? bodyText : bodyHtml }));
  }, [bodyText, bodyHtml, activeTab]);

  const handleSave = async () => {
    const data = {
      template_type: formData.template_type,
      subject: formData.subject,
      body: formData.body,
      body_format: formData.body_format,
      is_default: formData.is_default,
      attach_documents: formData.attach_documents,
      attach_audit_log: formData.attach_audit_log,
    };

    await onSave(data);
  };

  const handleTabChange = (event: React.SyntheticEvent, newValue: number) => {
    const newFormat = newValue === 0 ? 'text' : 'html';
    setActiveTab(newValue);
    setFormData({ ...formData, body_format: newFormat });
  };

  const getBodyPlaceholder = (templateType: string, isHtml: boolean) => {
    if (isHtml) {
      switch (templateType) {
        case 'invitation':
          return `<p>Hi <strong>{submitter.name}</strong>,</p>

                  <p>You have been invited to sign the "<strong>{template.name}</strong>" document.</p>

                  <p>Please click the button below to review and sign:</p>
                  <p><a href="{submitter.link}" style="background: #007bff; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; display: inline-block;">Review and Sign</a></p>

                  <p>If you have any questions, please reply to this email.</p>

                  <p>Best regards,<br>
                  {account.name}</p>`;
        case 'reminder':
          return `<p>Hi <strong>{submitter.name}</strong>,</p>

                  <p>This is a reminder that you have been invited to sign the "<strong>{template.name}</strong>" document.</p>

                  <p>Please click the button below to review and sign:</p>
                  <p><a href="{submitter.link}" style="background: #ff9800; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; display: inline-block;">Review and Sign</a></p>

                  <p>This is reminder #{reminder.number}.</p>

                  <p>Best regards,<br>
                  {account.name}</p>`;
        case 'copy':
          return `<p>Hi <strong>{submitter.name}</strong>,</p>

                  <p>Here is a copy of the completed document "<strong>{template.name}</strong>" that you signed.</p>

                  <p>The document has been successfully signed by all parties.</p>

                  <p>You can download the completed document from the attachment below.</p>

                  <p>Thank you for using our service!</p>

                  <p>Best regards,<br>
                  {account.name}</p>`;
        default:
          return '';
      }
    } else {
      switch (templateType) {
        case 'invitation':
          return `Hi {submitter.name},

                  You have been invited to sign the "{template.name}" document.

                  Please click the link below to review and sign:
                  {submitter.link}

                  If you have any questions, please reply to this email.

                  Best regards,
                  {account.name}`;
        case 'reminder':
          return `Hi {submitter.name},

                This is a reminder that you have been invited to sign the "{template.name}" document.

                Please click the link below to review and sign:
                {submitter.link}

                This is reminder #{reminder.number}.

                Best regards,
                {account.name}`;
        case 'copy':
          return `Hi {submitter.name},

                  Here is a copy of the completed document "{template.name}" that you signed.

                  The document has been successfully signed by all parties.

                  You can download the completed document from the attachment.

                  Thank you for using our service!

                  Best regards,
                  {account.name}`;
        default:
          return '';
      }
    }
  };

  return (
    <Dialog open={open} onClose={onClose} maxWidth="md" fullWidth>
      <DialogTitle>
        {editingTemplate ? 'Edit Email Template' : 'Create Email Template'}
      </DialogTitle>
      <DialogContent>
        <Box sx={{ mt: 2 }}>
          <FormControl fullWidth margin="normal" required>
            <InputLabel>Email Type</InputLabel>
            <Select
              value={formData.template_type}
              label="Email Type"
              onChange={(e) => setFormData({ ...formData, template_type: e.target.value })}
            >
              <MenuItem value="invitation">Invitation Email</MenuItem>
              <MenuItem value="reminder">Reminder Email</MenuItem>
              <MenuItem value="completion">Completion Email</MenuItem>
              <MenuItem value="copy">Copy Email</MenuItem>
            </Select>
          </FormControl>
          <TextField
            fullWidth
            label="Email Subject"
            value={formData.subject}
            onChange={(e) => setFormData({ ...formData, subject: e.target.value })}
            margin="normal"
            required
            placeholder={
              formData.template_type === 'invitation' ? 'Please sign: {template.name}' :
              formData.template_type === 'reminder' ? 'Reminder: Please sign {template.name}' :
              formData.template_type === 'completion' ? 'Document completed: {template.name}' :
              'Copy: {template.name}'
            }
          />

          <Box sx={{ mt: 2 }}>
           <Tabs
              value={activeTab}
              onChange={handleTabChange}
              textColor="inherit"
              TabIndicatorProps={{ style: { backgroundColor: '#fff' } }}
              sx={{
                  '& .MuiTab-root': {
                  color: '#fff',
                  },
                  '& .Mui-selected': {
                  color: '#fff',
                  fontWeight: 'bold'
                  }
              }}
              >
              <Tab label="Text" />
              <Tab label="HTML" />
              </Tabs>

            <Paper sx={{ mt: 1, p: 2 }}>
              {activeTab === 0 ? (
                <TextField
                  fullWidth
                  multiline
                  rows={10}
                  label="Email Body (Text)"
                  value={bodyText}
                  onChange={(e) => setBodyText(e.target.value)}
                  placeholder={getBodyPlaceholder(formData.template_type, false)}
                  helperText={
                    formData.template_type === 'invitation'
                      ? "Available variables: {submitter.name}, {template.name}, {submitter.link}, {account.name}"
                      : formData.template_type === 'reminder'
                      ? "Available variables: {submitter.name}, {template.name}, {submitter.link}, {account.name}, {reminder.number}"
                      : formData.template_type === 'completion'
                      ? "Available variables: {submitter.name}, {template.name}, {submitter.link}, {account.name}, {completed.signers}, {progress}"
                      : "Available variables: {submitter.name}, {template.name}, {account.name}"
                  }
                />
              ) : (
                <TextField
                  fullWidth
                  multiline
                  rows={10}
                  label="Email Body (HTML)"
                  value={bodyHtml}
                  onChange={(e) => setBodyHtml(e.target.value)}
                  placeholder={getBodyPlaceholder(formData.template_type, true)}
                  helperText={
                    formData.template_type === 'invitation'
                      ? "Available variables: {submitter.name}, {template.name}, {submitter.link}, {account.name}"
                      : formData.template_type === 'reminder'
                      ? "Available variables: {submitter.name}, {template.name}, {submitter.link}, {account.name}, {reminder.number}"
                      : formData.template_type === 'completion'
                      ? "Available variables: {submitter.name}, {template.name}, {submitter.link}, {account.name}, {completed.signers}, {progress}"
                      : "Available variables: {submitter.name}, {template.name}, {account.name}"
                  }
                />
              )}
            </Paper>
          </Box>

          <FormControlLabel
            control={
              <Switch
                checked={formData.is_default}
                onChange={(e) => setFormData({ ...formData, is_default: e.target.checked })}
              />
            }
            label="Set as default template"
            sx={{ mt: 2 }}
          />

          {(formData.template_type === 'completion' || formData.template_type === 'copy') && (
            <>
              <FormControlLabel
                control={
                  <Switch
                    checked={formData.attach_documents}
                    onChange={(e) => setFormData({ ...formData, attach_documents: e.target.checked })}
                  />
                }
                label="Attach documents: If enabled, the email will automatically attach the signed documents (original PDF with signatures) for recipients to download directly."
                sx={{ mt: 2 }}
              />
              <FormControlLabel
                control={
                  <Switch
                    checked={formData.attach_audit_log}
                    onChange={(e) => setFormData({ ...formData, attach_audit_log: e.target.checked })}
                  />
                }
                label="Attach audit log PDF: If enabled, the email will automatically attach a PDF audit log containing the history of signing actions, timestamps, and authentication information to ensure transparency and legal compliance."
                sx={{ mt: 2 }}
              />
            </>
          )}
        </Box>
      </DialogContent>
      <DialogActions color='white'>
         <Button
              onClick={onClose}
              variant="outlined"
              color="inherit"
              sx={{
                borderColor: "#475569",
                color: "#cbd5e1",
                textTransform: "none",
                fontWeight: 500,
                "&:hover": { backgroundColor: "#334155" },
              }}
          >
              Cancel
          </Button>
          <CreateTemplateButton text="Save" loading={loading} onClick={handleSave} />
      </DialogActions>
    </Dialog>
  );
}
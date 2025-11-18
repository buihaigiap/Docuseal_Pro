import { useState, useEffect } from 'react';
import {
    Box,Card,CardContent,Typography,Button,TextField,Dialog,
    DialogTitle,DialogContent,DialogActions,List,ListItem,ListItemText,ListItemSecondaryAction,
    IconButton,Switch,FormControlLabel,Tabs,Tab,Paper,FormControl,InputLabel,Select,MenuItem,
} from '@mui/material';
import {Plus,Edit,Trash2,Save,X} from 'lucide-react';
import upstashService from '../../../ConfigApi/upstashService';
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

export default function EmailTemplatesPage() {
  const [templates, setTemplates] = useState<EmailTemplate[]>([]);
  const [loading, setLoading] = useState(true);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingTemplate, setEditingTemplate] = useState<EmailTemplate | null>(null);
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
    fetchTemplates();
  }, []);

  useEffect(() => {
    setActiveTab(formData.body_format === 'html' ? 1 : 0);
  }, [formData.body_format]);

  useEffect(() => {
    setFormData(prev => ({ ...prev, body: activeTab === 0 ? bodyText : bodyHtml }));
  }, [bodyText, bodyHtml, activeTab]);

  const fetchTemplates = async () => {
    try {
      const res = await upstashService.getEmailTemplates();
      if (res.success) {
        setTemplates(res.data);
      }
    } catch (error) {
      console.error('Failed to fetch email templates:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCreate = () => {
    setEditingTemplate(null);
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
    setDialogOpen(true);
  };

  const handleEdit = (template: EmailTemplate) => {
    setEditingTemplate(template);
    setFormData({
      template_type: template.template_type,
      subject: template.subject,
      body: template.body,
      body_format: template.body_format,
      is_default: template.is_default,
      attach_documents: template.attach_documents || false,
      attach_audit_log: template.attach_audit_log || false,
    });
    setBodyText(template.body_format === 'text' ? template.body : '');
    setBodyHtml(template.body_format === 'html' ? template.body : '');
    setDialogOpen(true);
  };

  const handleDelete = async (id: number) => {
    if (!confirm('Are you sure you want to delete this email template?')) return;

    try {
      await upstashService.deleteEmailTemplate(id);
      setTemplates(templates.filter(t => t.id !== id));
    } catch (error) {
      console.error('Failed to delete email template:', error);
    }
  };

  const handleSave = async () => {
    try {
      const data = {
        template_type: formData.template_type,
        subject: formData.subject,
        body: formData.body,
        body_format: formData.body_format,
        is_default: formData.is_default,
        attach_documents: formData.attach_documents,
        attach_audit_log: formData.attach_audit_log,
      };

      if (editingTemplate) {
        await upstashService.updateEmailTemplate(editingTemplate.id, data);
        setTemplates(templates.map(t =>
          t.id === editingTemplate.id
            ? { ...t, ...data, updated_at: new Date().toISOString() }
            : t
        ));
      } else {
        const res = await upstashService.createEmailTemplate(data);
        if (res.success) {
          setTemplates([...templates, res.data]);
        }
      }
      setDialogOpen(false);
    } catch (error) {
      console.error('Failed to save email template:', error);
    }
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
        case 'completion':
          return `<p>Hi <strong>{account.name}</strong>,</p>

                  <p>The document "<strong>{template.name}</strong>" has been successfully signed by all parties.</p>

                  <p>You can download the completed document from your DocuSeal Pro dashboard.</p>

                  <p>Thank you for using our service!</p>

                  <p>Best regards,<br>
                  DocuSeal Pro</p>`;
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
        case 'completion':
          return `Hi {account.name},

                  The document "{template.name}" has been successfully signed by all parties.

                  You can download the completed document from your DocuSeal Pro dashboard.

                  Thank you for using our service!

                  Best regards,
                  DocuSeal Pro`;
        default:
          return '';
      }
    }
  };

  if (loading) {
    return (
      <Box display="flex" justifyContent="center" alignItems="center" minHeight="200px">
        <Typography>Loading...</Typography>
      </Box>
    );
  }

  return (
    <Box>
      <Box display="flex" justifyContent="space-between" alignItems="center" mb={3}>
        <Typography variant="h4" component="h1">
          Email Templates
        </Typography>
        <Button
          variant="contained"
          startIcon={<Plus />}
          onClick={handleCreate}
        >
          Create Template
        </Button>
      </Box>

      <Card>
        <CardContent>
          {templates.length === 0 ? (
            <Typography color="textSecondary" align="center" py={4}>
              No email templates found. Create your first template to get started.
            </Typography>
          ) : (
            <List>
              {templates.map((template) => (
                <ListItem key={template.id} divider>
                  <ListItemText
                   primary={
                            `${template.template_type === 'invitation'
                                ? 'Invitation'
                                : template.template_type === 'reminder'
                                ? 'Reminder'
                                : 'Completion'
                            }${template.is_default ? ' (Default)' : ''}`
                        }
                    secondary={
                      <>
                        <Typography variant="body2" color="textSecondary">
                          Subject: {template.subject}
                        </Typography>
                        <Typography variant="body2" color="textSecondary">
                          Format: {template.body_format === 'html' ? 'HTML' : 'Text'}
                        </Typography>
                      </>
                    }
                  />
                  <ListItemSecondaryAction color='white'>
                    <IconButton onClick={() => handleEdit(template)}>
                      <Edit  color='white'/>
                    </IconButton>
                    <IconButton onClick={() => handleDelete(template.id)}>
                      <Trash2 color='white'/>
                    </IconButton>
                  </ListItemSecondaryAction>
                </ListItem>
              ))}
            </List>
          )}
        </CardContent>
      </Card>

      <Dialog open={dialogOpen} onClose={() => setDialogOpen(false)} maxWidth="md" fullWidth>
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
                'Document completed: {template.name}'
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
                    helperText="Available variables: {submitter.name}, {template.name}, {submitter.link}, {account.name}, {reminder.number}"
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
                    helperText="Available variables: {submitter.name}, {template.name}, {submitter.link}, {account.name}, {reminder.number}"
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

            {formData.template_type === 'completion' && (
              <>
                <FormControlLabel
                  control={
                    <Switch
                      checked={formData.attach_documents}
                      onChange={(e) => setFormData({ ...formData, attach_documents: e.target.checked })}
                    />
                  }
                  label="Attach documents: Nếu bật, email sẽ tự động đính kèm các tài liệu đã ký (PDF gốc với chữ ký) để người nhận có thể tải xuống trực tiếp."
                  sx={{ mt: 2 }}
                />
                <FormControlLabel
                  control={
                    <Switch
                      checked={formData.attach_audit_log}
                      onChange={(e) => setFormData({ ...formData, attach_audit_log: e.target.checked })}
                    />
                  }
                  label="Attach audit log PDF: Nếu bật, email sẽ tự động đính kèm một tệp PDF ghi nhật ký kiểm tra (audit log), chứa lịch sử các hành động ký, thời gian, và thông tin xác thực để đảm bảo tính minh bạch và tuân thủ pháp lý."
                  sx={{ mt: 2 }}
                />
              </>
            )}
          </Box>
        </DialogContent>
        <DialogActions color='white'> 
           <Button
                onClick={() => setDialogOpen(false)}
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
    </Box>
  );
}
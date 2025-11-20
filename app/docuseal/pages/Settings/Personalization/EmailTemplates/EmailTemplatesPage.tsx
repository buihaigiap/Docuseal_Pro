import { useState, useEffect } from 'react';
import {
    Box,Card,CardContent,Typography,List,ListItem,ListItemText,ListItemSecondaryAction,
    IconButton,
} from '@mui/material';
import {Edit} from 'lucide-react';
import upstashService from '../../../../ConfigApi/upstashService';
import EmailTemplateDialog from './EmailTemplateDialog';

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
  useEffect(() => {
    fetchTemplates();
  }, []);

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

  const handleEdit = (template: EmailTemplate) => {
    setEditingTemplate(template);
    setDialogOpen(true);
  };

  const handleSave = async (data: any) => {
    try {
      if (editingTemplate) {
        await upstashService.updateEmailTemplate(editingTemplate.id, data);
        setTemplates(templates.map(t =>
          t.id === editingTemplate.id
            ? { ...t, ...data, updated_at: new Date().toISOString() }
            : t
        ));
      }
      setDialogOpen(false);
    } catch (error) {
      console.error('Failed to save email template:', error);
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
      </Box>

      <Card>
        <CardContent>
          {templates.length === 0 ? (
            <Typography color="textSecondary" align="center" py={4}>
              No email templates found. Default templates are created automatically when users register.
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
                                : template.template_type === 'completion'
                                ? 'Completion'
                                : 'Copy'
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
                  </ListItemSecondaryAction>
                </ListItem>
              ))}
            </List>
          )}
        </CardContent>
      </Card>

      <EmailTemplateDialog
        open={dialogOpen}
        onClose={() => setDialogOpen(false)}
        editingTemplate={editingTemplate}
        onSave={handleSave}
        loading={loading}
      />
    </Box>
  );
}
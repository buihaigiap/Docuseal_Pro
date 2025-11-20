import { useState } from 'react';
import {
    Typography, Card, CardContent, List, ListItem, ListItemText, ListItemSecondaryAction, IconButton,
} from '@mui/material';
import { Edit } from 'lucide-react';
import upstashService from '../../../ConfigApi/upstashService';
import toast from 'react-hot-toast';
import EmailTemplateDialog from './EmailTemplates/EmailTemplateDialog';

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

interface EmailTemplatesSectionProps {
  templates: EmailTemplate[];
  templatesLoading: boolean;
  fetchTemplates: () => void;
}

export default function EmailTemplatesSection({
  templates,
  templatesLoading,
  fetchTemplates
}: EmailTemplatesSectionProps) {
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingTemplate, setEditingTemplate] = useState<EmailTemplate | null>(null);

  const handleEditTemplate = (template: EmailTemplate) => {
    setEditingTemplate(template);
    setDialogOpen(true);
  };

  const handleSaveTemplate = async (data: any) => {
    try {
      if (editingTemplate) {
        await upstashService.updateEmailTemplate(editingTemplate.id, data);
        toast.success('Email template updated successfully');
      }
      setDialogOpen(false);
      setEditingTemplate(null);
      fetchTemplates();
    } catch (error) {
      console.error('Failed to save email template:', error);
      toast.error('Failed to save email template');
    }
  };

  return (
    <>
      <Card sx={{ mt: 3 }}>
        <CardContent>
          <Typography variant="h6" gutterBottom>
            Email Templates
          </Typography>
          <Typography variant="body2" color="textSecondary" gutterBottom>
            Customize email templates for invitations, reminders, and other notifications.
          </Typography>

          {templatesLoading ? (
            <Typography>Loading templates...</Typography>
          ) : (
            <List>
              {templates.map((template) => (
                <ListItem key={template.id} divider>
                  <ListItemText
                    primary={`${template.template_type.charAt(0).toUpperCase() + template.template_type.slice(1)} Template`}
                    secondary={template.subject}
                  />
                  <ListItemSecondaryAction>
                    <IconButton onClick={() => handleEditTemplate(template)}>
                      <Edit size={20} color='white'/>
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
        onClose={() => {
          setDialogOpen(false);
          setEditingTemplate(null);
        }}
        editingTemplate={editingTemplate}
        onSave={handleSaveTemplate}
        loading={false}
      />
    </>
  );
}
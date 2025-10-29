import React from 'react';
import { Dialog, DialogTitle, DialogContent, DialogActions, TextField, Button, Box, Typography } from '@mui/material';
import CreateTemplateButton from './CreateTemplateButton';

interface InviteModalProps {
  open: boolean;
  onClose: () => void;
  partnerEmails: Record<string, string>;
  onPartnerEmailsChange: (emails: Record<string, string>) => void;
  onSubmit: (e: React.FormEvent) => void;
  loading?: boolean;
}

const InviteModal: React.FC<InviteModalProps> = ({
  open,
  onClose,
  partnerEmails,
  onPartnerEmailsChange,
  onSubmit,
  loading = false
}) => {
  return (
    <Dialog
      open={open}
      onClose={onClose}
      maxWidth="sm"
      fullWidth
    >
      <form onSubmit={onSubmit}>
        <DialogTitle>Invite Signers</DialogTitle>
        <DialogContent>
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, pt: 1 }}>
            {Object.keys(partnerEmails).map((partner) => (
              <div key={partner}>
                <Typography variant="subtitle2" sx={{ mb: 1 }}>{partner}</Typography>
                <TextField
                  type="email"
                  placeholder="Email Address"
                  value={partnerEmails[partner]}
                  onChange={(e) => onPartnerEmailsChange({...partnerEmails, [partner]: e.target.value})}
                  required
                  fullWidth
                  size="small"
                />
              </div>
            ))}
          </Box>
        </DialogContent>
        <DialogActions>
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
          <CreateTemplateButton text="Send Invitations" loading={loading} />
        </DialogActions>
      </form>
    </Dialog>
  );
};

export default InviteModal;
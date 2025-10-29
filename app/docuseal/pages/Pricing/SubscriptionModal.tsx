import React from "react";
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  Typography,
  Box,
  Alert,
} from "@mui/material";
import { useAuth } from "../../contexts/AuthContext";

interface SubscriptionModalProps {
  isOpen: boolean;
  onClose: () => void;
  plan: any;
  error: string;
}

const SubscriptionModal: React.FC<SubscriptionModalProps> = ({
  isOpen,
  onClose,
  plan,
  error,
}) => {
  const { user } = useAuth();
  if (!plan) return null;

  return (
    <Dialog
      open={isOpen}
      onClose={onClose}
      maxWidth="sm"
      fullWidth
    >
      <DialogTitle sx={{ textAlign: "center", fontWeight: "bold", fontSize: 22 }}>
        Confirm subscription to {plan.name}
      </DialogTitle>

      <DialogContent>
        <Box textAlign="center" mt={2}>
          <Typography variant="h3" fontWeight="800" color="white">
            {plan.price.toLocaleString("en-US")}$
            <Typography
              component="span"
              variant="subtitle1"
              color="gray"
              sx={{ ml: 0.5 }}
            >
              /{plan.period}
            </Typography>
          </Typography>

          <Typography variant="body2" color="gray" mt={1}>
            You will be redirected to a secure payment gateway.
          </Typography>
        </Box>

        {error && (
          <Alert severity="error" sx={{ mt: 3, borderRadius: 2 }}>
            {error}
          </Alert>
        )}

        <Box
          mt={4}
          p={2}
          sx={{
            backgroundColor: "rgba(15,23,42,0.6)",
            border: "1px solid #334155", 
            borderRadius: 2,
          }}
        >
          <Typography variant="body2" color="gray.300">
            Once the payment is completed successfully, the {plan.name} plan will be
            activated immediately and you’ll have unlimited access to all features.
          </Typography>
        </Box>
      </DialogContent>

      <DialogActions sx={{ px: 3, pb: 3, justifyContent: "center", gap: 2 }}>
          <Button
            onClick={onClose}
            variant="outlined"
            color="inherit"
            sx={{
              borderColor: "#475569", // slate-600
              color: "#cbd5e1",
              textTransform: "none",
              fontWeight: 500,
              "&:hover": { backgroundColor: "#334155" },
            }}
        >
          Cancel
        </Button>
        <Button
          variant="contained"
          color="secondary"
          sx={{
            backgroundColor: "#7c3aed", // violet-600
            "&:hover": { backgroundColor: "#6d28d9" },
            textTransform: "none",
            fontWeight: 600,
          }}
          onClick={() => { window.open(`https://buy.stripe.com/test_fZubJ054m2PCe66f0A9bO00?client_reference_id=${user?.id || ''}`, '_blank') }}
        >
          Payment
        </Button>
      
      </DialogActions>
    </Dialog>
  );
};

export default SubscriptionModal;

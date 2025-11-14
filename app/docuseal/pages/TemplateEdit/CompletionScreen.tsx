import { Button, Typography } from '@mui/material';
import toast from 'react-hot-toast';
import upstashService from '../../ConfigApi/upstashService';
import CreateTemplateButtonProps from '../../components/CreateTemplateButton';
import { useNavigate } from 'react-router-dom';

interface CompletionScreenProps {
  signedDate: string;
  templateName?: string;
  token: string;
  allowResubmit: boolean;
}

const CompletionScreen: React.FC<CompletionScreenProps> = ({ 
  signedDate,
  templateName,
  token, 
  allowResubmit 
}) => {
  const navigate = useNavigate();

  const handleSendEmail = async () => {
    try {
      await upstashService.sendCopyEmail(token);
      toast.success('Email sent successfully');
    } catch (error) {
      toast.error('Failed to send email');
    }
  };

  const handleDownload = async () => {
    try {
      const response = await upstashService.downLoadFile(token);
      const blob = new Blob([response.data], { type: 'application/pdf' });
      const link = document.createElement('a');
      link.href = URL.createObjectURL(blob);
      link.download = `${templateName || 'signed_document'}.pdf`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(link.href);
      toast.success('Download started');
    } catch (error: any) {
      if (error.response?.status === 401 || error.status === 401) {
        // Authentication required, redirect to login with return URL
        const currentUrl = window.location.href;
        console.log('CompletionScreen 401 - current URL:', currentUrl);
        const loginUrl = `/login?redirect=${encodeURIComponent(currentUrl)}`;
        console.log('CompletionScreen 401 - redirecting to:', loginUrl);
        window.location.href = window.location.origin + loginUrl;
        return;
      }
      toast.error('Failed to download');
    }
  };

  const handleResubmit = async () => {
    try {
      await upstashService.resubmitSubmission(token);
      toast.success('Form reset successfully. You can now resubmit.');
      window.location.reload();
    } catch (error) {
      toast.error('Failed to reset form');
    }
  };

  return (
    <div className="flex items-center justify-center  ">
      <div className="max-w-md w-full  rounded-lg shadow-lg p-8">
        <div className=" mb-6">
            <div className="h-[200px]">
                <img src='/logo.png' alt="Logo"/>      
            </div>
          <Typography variant="body2" color="textSecondary" sx={{ mb: 1 }}>
            Template Name: {templateName}
          </Typography>
          <Typography variant="body2" color="textSecondary">
            Signed on {signedDate}
          </Typography>
        </div>

        <div className="space-y-3">
          <Button
            variant="contained"
            fullWidth
            sx={{
              textTransform: 'none',
              backgroundColor: '#4f46e5',
              '&:hover': { backgroundColor: '#4338ca' }
            }}
            onClick={handleSendEmail}
          >
            SEND COPY TO EMAIL
          </Button>

          <Button
            variant="outlined"
            fullWidth
            sx={{
              textTransform: 'none',
              borderColor: '#4f46e5',
              color: 'white',
            }}
            onClick={handleDownload}
          >
            DOWNLOAD DOCUMENTS
          </Button>

          {allowResubmit && (
            <CreateTemplateButtonProps
                text = "RESUBMIT FORM"
                onClick={handleResubmit}
                width = "100%"
            />
           
          )}
        </div>
      </div>
    </div>
  );
};

export default CompletionScreen;

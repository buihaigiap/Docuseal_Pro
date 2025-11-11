import { Button, Typography } from '@mui/material';
import toast from 'react-hot-toast';
import upstashService from '../../ConfigApi/upstashService';
import CreateTemplateButtonProps from '../../components/CreateTemplateButton';
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
  const handleSendEmail = async () => {
    try {
      await upstashService.sendCopyEmail(token);
      toast.success('Email sent successfully');
    } catch (error) {
      toast.error('Failed to send email');
    }
  };

  const handleDownload = () => {
    try {
      window.open(`/public/download/${token}`, '_blank');
      toast.success('Download started');
    } catch (error) {
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

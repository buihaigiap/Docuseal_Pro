import { Submitter } from '../../../types';
import upstashService from '../../../ConfigApi/upstashService';
import toast from 'react-hot-toast';
export const downloadSignedPDF = async (submitter: Submitter, pdfUrl?: string) => {
  try {
    const response = await upstashService.downLoadFile(submitter.token);
    const blob = new Blob([response.data], { type: 'application/pdf' });
    const link = document.createElement('a');
    link.href = URL.createObjectURL(blob);
    link.download = 'signed_document.pdf';  // Or use filename from backend header if available
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(link.href);
    toast.success('PDF downloaded successfully!');
  } catch (err: any) {
    console.error('Download error:', err);
    toast.error(`Failed to download PDF: ${err.message}`);
  }
};
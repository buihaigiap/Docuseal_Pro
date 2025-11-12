import React from 'react';
import { downloadSignedPDF } from '../services/pdfDownloadService';
import upstashService from '../ConfigApi/upstashService';
import { useBasicSettings } from '../hooks/useBasicSettings';
import toast from 'react-hot-toast';
import { Submitter } from '../types';

// Example: How to implement onDownload in the parent component of SubmitterItem
// This would go in the component that renders SubmitterItem (e.g., TemplateDetailPage)

const handleSubmitterDownload = async (submitter: Submitter, pdfUrl?: string) => {
  try {
    // 1. Fetch submission signatures data for this submitter
    const signaturesResult = await upstashService.getSubmissionSignatures(submitter.token);

    if (!signaturesResult.success) {
      throw new Error(signaturesResult.message || 'Failed to fetch signatures');
    }

    // 2. Get submitter info (you might need to fetch this separately or have it available)
    const submitterInfo = {
      id: submitter.id,
      email: submitter.email
    };

    // 3. Get global settings (this would be from the hook in the actual component)
    // const { globalSettings } = useBasicSettings();

    // For this example, we'll use a mock globalSettings
    const globalSettings = {
      add_signature_id_to_the_documents: true,
      require_signing_reason: false
    };

    // 4. Call the shared PDF download service
    await downloadSignedPDF(
      pdfUrl || signaturesResult.data.template_info.document.url, // Use provided pdfUrl or get from signatures data
      signaturesResult.data.bulk_signatures,
      signaturesResult.data.template_info.name,
      submitterInfo,
      globalSettings
    );

    toast.success('PDF downloaded successfully!');
  } catch (err: any) {
    console.error('Download error:', err);
    toast.error(`Failed to download PDF: ${err.message || 'Unknown error'}`);
  }
};

// Example usage in a parent component:
/*
const ParentComponent = () => {
  const { globalSettings } = useBasicSettings();

  return (
    <SubmitterItem
      party={party}
      signatureType={signatureType}
      overallStatus={overallStatus}
      onDownload={handleSubmitterDownload}
      pdfUrl={pdfUrl}
    />
  );
};
*/

export { handleSubmitterDownload };
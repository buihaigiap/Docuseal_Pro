import React, { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { SubmissionSignaturesResponse } from '../types';
import PdfViewer from '../components/PdfViewer';
import upstashService from '../ConfigApi/upstashService';

const SignedSubmissionPage = () => {
  const { token } = useParams<{ token: string }>();
  const navigate = useNavigate();
  const [data, setData] = useState<SubmissionSignaturesResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [isMobile, setIsMobile] = useState(window.innerWidth < 768);
  useEffect(() => {
    const handleResize = () => setIsMobile(window.innerWidth < 768);
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);
  const [submitterInfo, setSubmitterInfo] = useState<{ id: number; email: string } | null>(null);
  useEffect(() => {
    const fetchData = async () => {
      try {
        // Fetch both signatures and fields data in parallel
        const [signaturesResult, fieldsResult] = await Promise.all([
          upstashService.getSubmissionSignatures(token),
          upstashService.getSubmissionFields(token)
        ]);

        console.log('Signatures Result:', signaturesResult);
        console.log('Fields Result:', fieldsResult);

        if (signaturesResult.success) {
          setData(signaturesResult.data);
        } else {
          setError(signaturesResult.message || 'Failed to fetch signatures data');
        }

        if (fieldsResult.success && fieldsResult.data.information) {
          setSubmitterInfo({
            id: fieldsResult.data.information.id,
            email: fieldsResult.data.information.email
          });
        }
      } catch (err) {
        console.error('Fetch error:', err);
        setError('An error occurred while fetching data');
      } finally {
        setLoading(false);
      }
    };

    if (token) {
      fetchData();
    }
  }, [token]);

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-500 mx-auto mb-4"></div>
          <p>Loading submission...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
        <div className="text-center">
          <p className="text-red-500 mb-4">{error}</p>
          <button onClick={() => navigate(-1)} className="px-4 py-2 bg-indigo-600 rounded-md hover:bg-indigo-700">
            Go Back
          </button>
        </div>
      </div>
    );
  }

  if (!data) return null;

  return (
      <div className={`${isMobile ? 'relative' : 'grid grid-cols-1 lg:grid-cols-3 gap-6'}`}>
        <div className={`${isMobile ? 'w-full' : 'lg:col-span-2'}`}>
          <PdfViewer
            filePath={data.template_info.document.url}
            fields={data?.bulk_signatures?.map(sig => ({ ...sig.field_info, signature_value: sig.signature_value, reason: sig.reason }))}
            submitterId={submitterInfo?.id}
            submitterEmail={submitterInfo?.email}
            // scale={1.5}
          />
        </div>
      </div>
  );
};

export default SignedSubmissionPage;
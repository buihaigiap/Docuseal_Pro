import { useState } from 'react';
import upstashService from '../ConfigApi/upstashService';
import toast from 'react-hot-toast';

interface UseFileUploadOptions {
  maxSize?: number; // in bytes, default 10MB
  onUploadStart?: () => void;
  onUploadComplete?: (url: string) => void;
  onUploadError?: (error: Error) => void;
  onDeleteComplete?: () => void;
  onDeleteError?: (error: Error) => void;
}

interface UseFileUploadReturn {
  uploading: boolean;
  deleting: boolean;
  progress: number;
  uploadFile: (file: File) => Promise<string | null>;
  deleteFile: (fileUrl: string) => Promise<boolean>;
}

export const useFileUpload = (options: UseFileUploadOptions = {}): UseFileUploadReturn => {
  const {
    maxSize = 10 * 1024 * 1024, // 10MB default
    onUploadStart,
    onUploadComplete,
    onUploadError,
    onDeleteComplete,
    onDeleteError,
  } = options;

  const [uploading, setUploading] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [progress, setProgress] = useState(0);

  const uploadFile = async (file: File): Promise<string | null> => {
    // Validate file size
    if (file.size > maxSize) {
      const errorMsg = `File is too large. Maximum size is ${maxSize / (1024 * 1024)}MB. Current file: ${(file.size / (1024 * 1024)).toFixed(2)}MB.`;
      toast.error(errorMsg);
      if (onUploadError) {
        onUploadError(new Error(errorMsg));
      }
      return null;
    }

    try {
      setUploading(true);
      setProgress(0);
      
      if (onUploadStart) {
        onUploadStart();
      }

      const formData = new FormData();
      formData.append('file', file);

      const response = await upstashService.uploadPublicFile(formData, (progressEvent) => {
        if (progressEvent.total) {
          const percentCompleted = Math.round((progressEvent.loaded * 100) / progressEvent.total);
          setProgress(Math.min(percentCompleted, 99));
        }
      });

      const data = response.data;
      if (data && data.success && data.data && data.data.url) {
        setProgress(100);
        const url = data.data.url;
        
        if (onUploadComplete) {
          onUploadComplete(url);
        }
        
        toast.success('File uploaded successfully!');
        return url;
      } else {
        const errorMsg = data?.message || 'Upload failed. Please try again.';
        console.error('Upload failed: Invalid response format', data);
        toast.error(errorMsg);
        
        if (onUploadError) {
          onUploadError(new Error(errorMsg));
        }
        
        return null;
      }
    } catch (error: any) {
      console.error('Upload error:', error);
      
      let errorMsg = 'Unknown error. Please try again.';
      
      if (error.response) {
        console.error('Response status:', error.response.status);
        console.error('Response data:', error.response.data);
        errorMsg = error.response.data?.message || 'Error uploading file. Please try again.';
      } else if (error.request) {
        console.error('No response received:', error.request);
        errorMsg = 'No response from server. Please check your connection.';
      } else {
        console.error('Error message:', error.message);
        errorMsg = error.message;
      }
      
      toast.error(errorMsg);
      
      if (onUploadError) {
        onUploadError(error);
      }
      
      return null;
    } finally {
      setTimeout(() => {
        setUploading(false);
        setProgress(0);
      }, 300);
    }
  };

  const deleteFile = async (fileUrl: string): Promise<boolean> => {
    if (!fileUrl) {
      return false;
    }

    try {
      setDeleting(true);
      
      const response = await upstashService.deletePublicFile(fileUrl);
      const data = response.data;
      
      if (data && data.success) {
        toast.success('File deleted successfully!');
        
        if (onDeleteComplete) {
          onDeleteComplete();
        }
        
        return true;
      } else {
        const errorMsg = data?.message || 'Failed to delete file. Please try again.';
        console.error('Delete failed:', data);
        toast.error(errorMsg);
        
        if (onDeleteError) {
          onDeleteError(new Error(errorMsg));
        }
        
        return false;
      }
    } catch (error: any) {
      console.error('Delete error:', error);
      
      let errorMsg = 'Error deleting file. Please try again.';
      
      if (error.response) {
        console.error('Response status:', error.response.status);
        console.error('Response data:', error.response.data);
        errorMsg = error.response.data?.message || errorMsg;
      }
      
      toast.error(errorMsg);
      
      if (onDeleteError) {
        onDeleteError(error);
      }
      
      return false;
    } finally {
      setDeleting(false);
    }
  };

  return {
    uploading,
    deleting,
    progress,
    uploadFile,
    deleteFile,
  };
};

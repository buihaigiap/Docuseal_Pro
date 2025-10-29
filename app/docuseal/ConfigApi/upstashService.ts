import axiosClient from "./axiosClient";
import axios from "axios";

const JWT_LOCAL_STORAGE_KEY = 'token';

const upstashService = {
    // Auth APIs
    Login: async (data: any): Promise<any> => {
        const url = '/api/auth/login';
        return await axiosClient.post(url, data)
    },
    Register: async (data: any): Promise<any> => {
        const url = '/api/auth/register';
        return await axiosClient.post(url, data)
    },
    getMe: async (): Promise<any> => {
        const url = '/api/me';
        return await axiosClient.get(url)
    },
    changePassword: async (data: any): Promise<any> => {
        const url = '/api/auth/change-password';
        return await axiosClient.post(url, data)
    },
    forgotPassword: async (data: any): Promise<any> => {
        const url = '/api/auth/forgot-password';
        return await axiosClient.post(url, data)
    },
    resetPassword: async (data: any): Promise<any> => {
        const url = '/api/auth/reset-password';
        return await axiosClient.post(url, data)
    },

    updateProfile: async (data: any): Promise<any> => {
        const url = '/api/auth/profile';
        return await axiosClient.put(url, data)
    },
    // Template APIs
    getTemplates: async (): Promise<any> => {
        const url = '/api/templates';
        return await axiosClient.get(url)
    },
    createTemplateFromFile: async (data: any): Promise<any> => {
        const url = '/api/templates/from-file';
        return await axiosClient.post(url, data)
    },
    getTemplateFullInfo: async (id: number): Promise<any> => {
        const url = `/api/templates/${id}/full-info`;
        return await axiosClient.get(url)
    },
    cloneTemplate: async (id: any): Promise<any> => {
        const url = `/api/templates/${id}/clone`;
        return await axiosClient.post(url, {})
    },
    deleteTemplate: async (id: any): Promise<any> => {
        const url = `/api/templates/${id}`;
        return await axiosClient.delete(url)
    },

    // File APIs
    uploadFile: async (formData: FormData): Promise<any> => {
        const url = '/api/files/upload';
        return await axiosClient.post(url, formData, {
            headers: { 'Content-Type': 'multipart/form-data' }
        })
    },
    uploadPublicFile: async (formData: FormData): Promise<any> => {
        const url = '/api/files/upload/public';
        return await axiosClient.post(url, formData, {
            headers: { 'Content-Type': 'multipart/form-data' }
        })
    },
    previewFile: async (url: string) => {
        // Use axios directly to bypass response interceptor that strips headers
        const fullUrl = `${axiosClient.defaults.baseURL}/api/files/preview/${url}`;
        const config = {
            headers: {
                'Authorization': localStorage.getItem(JWT_LOCAL_STORAGE_KEY) ? `Bearer ${localStorage.getItem(JWT_LOCAL_STORAGE_KEY)}` : undefined
            },
            responseType: 'json' as const
        };
        return await axios.get(fullUrl, config);
    },
    downLoadFile: async (token: string) => {
        const apiUrl = `${axiosClient.defaults.baseURL}/public/download/${token}`;
        const config = {
            headers: {
                'Authorization': localStorage.getItem(JWT_LOCAL_STORAGE_KEY) ? `Bearer ${localStorage.getItem(JWT_LOCAL_STORAGE_KEY)}` : undefined
            },
            responseType: 'blob' as const
        };
        return await axios.get(apiUrl, config);
    },

    // Submission APIs
    createSubmission: async (data: any): Promise<any> => {
        const url = '/api/submissions';
        return await axiosClient.post(url, data)
    },
    deleteSubmitter: async (submitterId: number): Promise<any> => {
        const url = `/api/submitters/${submitterId}`;
        return await axiosClient.delete(url)
    },

    // Submission/Signatures APIs
    getSubmissionSignatures: async (token: string): Promise<any> => {
        const url = `/public/submissions/${token}/signatures`;
        return await axiosClient.get(url)
    },
    getSubmitterInfo: async (signingToken: string): Promise<any> => {
        const url = `/public/submitters/${signingToken}`;
        return await axiosClient.get(url)
    },
    getSubmissionFields: async (token: string): Promise<any> => {
        const url = `/public/submissions/${token}/fields`;
        return await axiosClient.get(url)
    },
    bulkSign: async (token: string, data: any): Promise<any> => {
        const url = `/public/signatures/bulk/${token}`;
        return await axiosClient.post(url, data)
    },

    // Field APIs
    createField: async (templateId: number, data: any): Promise<any> => {
        const url = `/api/templates/${templateId}/fields`;
        return await axiosClient.post(url, data)
    },
    updateField: async (templateId: number, fieldId: number, data: any): Promise<any> => {
        const url = `/api/templates/${templateId}/fields/${fieldId}`;
        return await axiosClient.put(url, data)
    },
    deleteField: async (templateId: number, fieldId: number): Promise<any> => {
        const url = `/api/templates/${templateId}/fields/${fieldId}`;
        return await axiosClient.delete(url)
    },

    // Folder APIs
    getFolders: async (): Promise<any> => {
        const url = '/api/folders';
        return await axiosClient.get(url)
    },
    getFolderTemplates: async (folderId: number): Promise<any> => {
        const url = `/api/folders/${folderId}/templates`;
        return await axiosClient.get(url)
    },
    moveTemplate: async (data: any): Promise<any> => {
        const url = '/api/folders';
        return await axiosClient.post(url, data)
    },
    updateFolder: async (folderId: number, data: any): Promise<any> => {
        const url = `/api/folders/${folderId}`;
        return await axiosClient.put(url, data)
    },
    moveTemplatePut: async (template_id: any, parent_folder_id: any): Promise<any> => {
        const url = `/api/templates/${template_id}/move/${parent_folder_id}`;
        return await axiosClient.put(url)
    },
    deleteFolder: async (folderId: number): Promise<any> => {
        const url = `/api/folders/${folderId}`;
        return await axiosClient.delete(url)
    },
    // Team APIs can be added here
    addTeam : async (data: any): Promise<any> => {
        const url = '/api/auth/users';
        return await axiosClient.post(url, data)
    },
    activateAccount : async (data: any): Promise<any> => {
        const url = '/api/auth/activate';
        return await axiosClient.post(url, data)
    },
    getUserAccounts : async (): Promise<any> => {
        const url = '/api/admin/members';
        return await axiosClient.get(url)
    }


}
export default upstashService

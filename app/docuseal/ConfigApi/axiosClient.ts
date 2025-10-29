import queryString from 'query-string';
import axios from "axios";

const JWT_LOCAL_STORAGE_KEY = 'token';
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || '';

const axiosClient = axios.create({
    baseURL: API_BASE_URL,
    headers: {
        'content-type': 'application/json',
        'Authorization': `Bearer ${JWT_LOCAL_STORAGE_KEY}`
    },
    paramsSerializer: params => queryString.stringify(params),
});


axiosClient.interceptors.request.use((config) => {
    const accessToken = localStorage.getItem(JWT_LOCAL_STORAGE_KEY);
    if (accessToken) {
        config.headers.Authorization = `Bearer ${accessToken}`;
    }
    return config;
})

axiosClient.interceptors.response.use((response) => {
    if (response && response.data) {
        if (response.data.status === '400') {
            throw response.data;
        }
        return response.data;
    }
    return response;
}, (error) => {
    if (error.data) {
        return error.data;
    }
    throw error;
});
export default axiosClient;

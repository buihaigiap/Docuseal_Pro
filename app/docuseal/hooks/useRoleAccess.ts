import { useAuth } from '../contexts/AuthContext';

export const useRoleAccess = (allowedRoles: string[]): boolean => {
  const { user } = useAuth();
  return user ? allowedRoles.includes(user.role) : false;
};

 export const canTemplate = (template: any) => {
     const { user } = useAuth();
    if (!user) return false;
    if (user.role === 'member') {
      return template?.user_id === user?.id;
    }
    return true; 
  };
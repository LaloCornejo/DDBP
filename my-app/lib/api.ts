// API client for Rust backend integration

// Base URL configuration
const API_BASE_URL = "http://localhost:8000";

// Types for API responses
export interface User {
  id: string;
  name: string;
  username: string;
  email: string;
  image: string;
  bio: string;
  postCount: number;
}

export interface Post {
  id: string;
  title: string;
  content: string;
  authorId: string;
  commentCount: number;
  timestamp: string;
  author?: {
    id: string;
    name: string;
    image: string;
  };
}

export interface Comment {
  id: string;
  postId: string;
  content: string;
  authorId: string;
  timestamp: string;
  author?: {
    id: string;
    name: string;
    image: string;
  };
}

export interface PaginatedResponse<T> {
  data: T[];
  hasMore: boolean;
  total?: number;
  page?: number;
  limit?: number;
}

// Error class for API requests
export class ApiError extends Error {
  status: number;
  
  constructor(message: string, status: number) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
  }
}

// Generic fetch function with error handling
async function fetchApi<T>(
  endpoint: string, 
  options: RequestInit = {}
): Promise<T> {
  const url = `${API_BASE_URL}${endpoint}`;
  
  try {
    const response = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ message: 'Unknown error' }));
      throw new ApiError(
        errorData.message || `API request failed with status ${response.status}`,
        response.status
      );
    }

    return await response.json() as T;
  } catch (error) {
    if (error instanceof ApiError) {
      throw error;
    }
    throw new ApiError(`Failed to fetch from ${endpoint}: ${(error as Error).message}`, 500);
  }
}

// API functions that mirror the mock data functions
export async function getUserById(id: string): Promise<User> {
  return fetchApi<User>(`/api/users/${id}`);
}

export async function getPostById(id: string): Promise<Post> {
  return fetchApi<Post>(`/api/posts/${id}`);
}

export async function getCommentsByPostId(postId: string): Promise<Comment[]> {
  return fetchApi<Comment[]>(`/api/comments/post/${postId}`);
}

export async function getPostsByAuthorId(authorId: string, page = 1, limit = 10): Promise<PaginatedResponse<Post>> {
  return fetchApi<PaginatedResponse<Post>>(`/api/users/posts/${authorId}?page=${page}&limit=${limit}`);
}

export async function getPaginatedPosts(page = 1, limit = 10): Promise<PaginatedResponse<Post>> {
  const response = await fetchApi<PaginatedResponse<Post>>(`/api/posts?page=${page}&limit=${limit}`);
  
  // If the API doesn't return author information embedded in posts, fetch it separately
  const enrichedPosts = await Promise.all(
    response.data.map(async (post) => {
      if (!post.author) {
        const author = await getUserById(post.authorId);
        return {
          ...post,
          author: {
            id: author.id,
            name: author.name,
            image: author.image
          }
        };
      }
      return post;
    })
  );
  
  return {
    ...response,
    data: enrichedPosts
  };
}

export async function enrichPostWithDetails(post: Post): Promise<Post & { comments: Comment[] }> {
  // Fetch author information if not already included
  let authorInfo = post.author;
  if (!authorInfo) {
    const author = await getUserById(post.authorId);
    authorInfo = {
      id: author.id,
      name: author.name,
      image: author.image
    };
  }
  
  // Fetch comments and their authors
  const comments = await getCommentsByPostId(post.id);
  const enrichedComments = await Promise.all(
    comments.map(async (comment) => {
      if (!comment.author) {
        const commentAuthor = await getUserById(comment.authorId);
        return {
          ...comment,
          author: {
            id: commentAuthor.id,
            name: commentAuthor.name,
            image: commentAuthor.image
          }
        };
      }
      return comment;
    })
  );
  
  return {
    ...post,
    author: authorInfo,
    comments: enrichedComments
  };
}

// Additional API functions for profile pages
export async function getFollowing(userId: string): Promise<User[]> {
  return fetchApi<User[]>(`/api/users/following/${userId}`);
}

export async function getFollowers(userId: string): Promise<User[]> {
  return fetchApi<User[]>(`/api/users/followers/${userId}`);
}

export async function getCommentsByUserId(userId: string): Promise<Comment[]> {
  return fetchApi<Comment[]>(`/api/comments/user/${userId}`);
}


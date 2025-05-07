import { PostCard } from "@/components/ui/post-card";

export const metadata = {
  title: "Social Feed - Home",
  description: "View the latest posts from our community",
  openGraph: {
    title: "Social Feed - Home",
    description: "View the latest posts from our community",
    type: "website",
    siteName: "Social Feed"
  }
};

// Define types to match our API responses
type ApiResponse<T> = {
  status: string;
  message: string;
  data: T[];
};

type User = {
  _id: string;
  username: string;
  email: string;
  bio: string;
  profile_picture_url: string;
  join_date: string;
};

type Comment = {
  _id: string;
  post_id: string;
  user_id: string;
  content: string;
};

type Post = {
  _id: string;
  user_id: string;
  title: string;
  content: string;
  media_urls?: string[];
  post_type?: string;
  like_count?: number;
  comment_count?: number;
};

// Fetch data from API endpoint
async function fetchDataFromApi<T>(endpoint: string): Promise<T[]> {
  const response = await fetch(`http://localhost:8000/api/${endpoint}`);
  if (!response.ok) {
    return [];
  }
  const apiResponse: ApiResponse<T> = await response.json();
  return apiResponse.data || [];
}

export default async function Home() {
  // Fetch data from our Rust backend
  const posts: Post[] = await fetchDataFromApi<Post>('posts');
  const users: User[] = await fetchDataFromApi<User>('users');
  const comments: Comment[] = await fetchDataFromApi<Comment>('comments');

  // Create a map of user IDs to users for easy lookup
  const userMap = users.reduce((map, user) => {
    map[user._id] = user;
    return map;
  }, {} as Record<string, User>);

  // Count comments for each post
  const commentCountByPost = comments.reduce((counts, comment) => {
    counts[comment.post_id] = (counts[comment.post_id] || 0) + 1;
    return counts;
  }, {} as Record<string, number>);
  
  // Format posts to match what our component expects
  const formattedPosts = posts.map(post => ({
    id: post._id,
    title: post.title || 'Post', // Use post_type as title or default to 'Post'
    content: post.content,
    author: {
      id: post.user_id,
      name: userMap[post.user_id]?.username || 'Unknown User',
      image: userMap[post.user_id]?.profile_picture_url || ''
    },
    commentCount: post.comment_count || commentCountByPost[post._id] || 0,
    timestamp: post.created_at || new Date().toISOString(),
  }));
  
  return (
    <div className="container mx-auto px-4 py-8">
      <header className="mb-8">
        <h1 className="text-3xl font-bold" id="main-heading">Latest Posts</h1>
        <p className="text-muted-foreground">Discover interesting content from our community</p>
      </header>
      
      <main aria-labelledby="main-heading">
        <div className="space-y-6">
          {formattedPosts.map((post) => (
            <PostCard
              key={post.id}
              id={post.id}
              title={post.title}
              content={post.content}
              author={post.author}
              commentCount={post.commentCount}
            />
          ))}
          
          {formattedPosts.length === 0 && (
            <div className="text-center p-12 border rounded-lg">
              <p className="text-muted-foreground">No posts found</p>
            </div>
          )}
        </div>
      </main>
    </div>
  );
}

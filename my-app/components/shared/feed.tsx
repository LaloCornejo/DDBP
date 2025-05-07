"use client";

import { useState, useEffect } from "react";
import { PostCard } from "@/components/ui/post-card";
import { Button } from "@/components/ui/button";
import { LoadingSkeleton } from "./loading-skeleton";
import { getPaginatedPosts, ApiError, Post as ApiPost } from "@/lib/api";

// Use our API Post type but ensure it has all required fields for the component
type Post = {
  id: string;
  title: string;
  content: string;
  author: {
    id: string;
    name: string;
    image: string;
  };
  commentCount: number;
  timestamp: string;
};

interface FeedProps {
  initialPosts: Post[];
  className?: string;
}

export function Feed({ initialPosts, className }: FeedProps) {
  const [posts, setPosts] = useState<Post[]>(initialPosts);
  const [page, setPage] = useState(1);
  const [loading, setLoading] = useState(false);
  const [hasMore, setHasMore] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadMorePosts = async () => {
    if (loading || !hasMore) return;
    
    setLoading(true);
    setError(null);
    
    try {
      const nextPage = page + 1;
      const response = await getPaginatedPosts(nextPage);
      
      if (response.data.length === 0) {
        setHasMore(false);
      } else {
        setPosts([...posts, ...response.data]);
        setPage(nextPage);
        setHasMore(response.hasMore);
      }
    } catch (error) {
      console.error("Failed to load more posts:", error);
      setError(error instanceof ApiError 
        ? `Error loading posts: ${error.message}` 
        : "Failed to load posts. Please try again.");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className={className}>
      <div className="space-y-6">
        {posts.map((post) => (
          <PostCard
            key={post.id}
            id={post.id}
            title={post.title}
            content={post.content}
            author={post.author}
            commentCount={post.commentCount}
          />
        ))}
        
        {loading && <LoadingSkeleton count={3} />}
        
        {hasMore && (
          <div className="mt-6 flex justify-center">
            <Button 
              onClick={loadMorePosts} 
              disabled={loading}
              variant="outline"
              aria-live="polite"
              aria-busy={loading}
            >
              {loading ? "Loading..." : "Load More"}
            </Button>
          </div>
        )}
        
        {!hasMore && posts.length > 0 && (
          <p className="text-center text-muted-foreground mt-6">
            No more posts to load
          </p>
        )}
        
        {posts.length === 0 && !loading && (
          <div className="text-center p-12 border rounded-lg">
            <p className="text-muted-foreground">No posts found</p>
          </div>
        )}
        
        {error && (
          <div className="p-4 mt-6 text-center border border-destructive/50 rounded-lg bg-destructive/10">
            <p className="text-destructive">{error}</p>
            <Button 
              onClick={() => {
                setError(null);
                loadMorePosts();
              }}
              variant="outline"
              className="mt-2"
            >
              Retry
            </Button>
          </div>
        )}
      </div>
    </div>
  );
}


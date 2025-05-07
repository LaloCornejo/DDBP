
import { Suspense } from "react";
import Link from "next/link";
import { notFound } from "next/navigation";
import { UserInfo } from "@/components/ui/user-info";
import { PostCard } from "@/components/ui/post-card";
import { LoadingSkeleton } from "@/components/shared/loading-skeleton";
import { getUserById, getPostsByAuthorId, getFollowers, getFollowing, ApiError } from "@/lib/api";
import { Button } from "@/components/ui/button";

interface UserPageProps {
  params: {
    id: string;
  };
}

export async function generateMetadata({ params }: UserPageProps) {
  try {
    const user = await getUserById(params.id);
    
    return {
      title: `${user.name}'s Profile - Social Feed`,
      description: `View ${user.name}'s profile and posts`,
      openGraph: {
        title: `${user.name}'s Profile - Social Feed`,
        description: `View ${user.name}'s profile and posts`,
        type: "profile",
        siteName: "Social Feed",
        images: [
          {
            url: user.image || "",
            width: 200,
            height: 200,
            alt: user.name
          }
        ]
      }
    };
  } catch (error) {
    return {
      title: "User Not Found",
      description: "The requested user profile could not be found.",
    };
  }
  
}

async function UserProfile({ userId }: { userId: string }) {
  try {
    const user = await getUserById(userId);
    const [followers, following] = await Promise.all([
      getFollowers(userId),
      getFollowing(userId)
    ]);
    
    return (
      <div>
        <UserInfo 
          user={user} 
          followersCount={followers.length}
          followingCount={following.length}
        />
      </div>
    );
  } catch (error) {
    return (
      <div className="p-4 text-center border border-destructive/50 rounded-lg bg-destructive/10">
        <p className="text-destructive">
          {error instanceof ApiError 
            ? `Error loading user profile: ${error.message}` 
            : "Failed to load user profile. Please try again."}
        </p>
      </div>
    );
  }
}

'use client';

import React, { useEffect, useState, useCallback } from "react";

// Define post type for consistency
type FormattedPost = {
  id: string;
  title: string;
  content: string;
  author: {
    id: string;
    name: string;
    image: string;
  };
  commentCount: number;
};

function UserPosts({ userId }: { userId: string }) {
  const [posts, setPosts] = useState<FormattedPost[]>([]);
  const [loading, setLoading] = useState(true);
  const [paginationLoading, setPaginationLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [page, setPage] = useState(1);
  const [hasMore, setHasMore] = useState(false);

  // Format posts for display
  const formatPosts = useCallback((apiPosts: any[]) => {
    return apiPosts.map(post => ({
      id: post.id,
      title: post.title,
      content: post.content.substring(0, 200) + (post.content.length > 200 ? '...' : ''),
      author: post.author || {
        id: post.authorId,
        name: "Unknown", // This should be populated by the API
        image: ""
      },
      commentCount: post.commentCount
    }));
  }, []);

  // Initial data loading
  useEffect(() => {
    async function loadInitialPosts() {
      setLoading(true);
      setError(null);
      
      try {
        const response = await getPostsByAuthorId(userId);
        setPosts(formatPosts(response.data));
        setHasMore(response.hasMore);
      } catch (err) {
        console.error("Failed to load user posts:", err);
        setError(
          err instanceof ApiError 
            ? `Error loading posts: ${err.message}` 
            : "Failed to load user posts. Please try again."
        );
      } finally {
        setLoading(false);
      }
    }

    loadInitialPosts();
  }, [userId, formatPosts]);

  // Function to load more posts
  const loadMorePosts = useCallback(async () => {
    if (paginationLoading || !hasMore) return;
    
    setPaginationLoading(true);
    setError(null);
    
    try {
      const nextPage = page + 1;
      const response = await getPostsByAuthorId(userId, nextPage);
      
      const newPosts = formatPosts(response.data);
      setPosts(prevPosts => [...prevPosts, ...newPosts]);
      setPage(nextPage);
      setHasMore(response.hasMore);
    } catch (err) {
      console.error("Failed to load more posts:", err);
      setError(
        err instanceof ApiError 
          ? `Error loading more posts: ${err.message}` 
          : "Failed to load more posts. Please try again."
      );
    } finally {
      setPaginationLoading(false);
    }
  }, [paginationLoading, hasMore, page, userId, formatPosts]);

  // Display loading state
  if (loading) {
    return <div aria-live="polite" aria-busy="true"><LoadingSkeleton count={3} /></div>;
  }

  // Display error state
  if (error) {
    return (
      <div className="p-4 text-center border border-destructive/50 rounded-lg bg-destructive/10">
        <p className="text-destructive">{error}</p>
        <Button 
          onClick={() => {
            setError(null);
            setPage(1);
            setPaginationLoading(false);
            // Reload initial posts
            setLoading(true);
            getPostsByAuthorId(userId)
              .then(response => {
                setPosts(formatPosts(response.data));
                setHasMore(response.hasMore);
              })
              .catch(err => {
                setError(
                  err instanceof ApiError 
                    ? `Error loading posts: ${err.message}` 
                    : "Failed to load posts. Please try again."
                );
              })
              .finally(() => setLoading(false));
          }}
          variant="outline"
          className="mt-2"
        >
          Retry
        </Button>
      </div>
    );
  }

  // No posts state
  if (posts.length === 0) {
    return <p className="text-muted-foreground">This user hasn't published any posts yet.</p>;
  }

  // Render posts with pagination
  return (
    <section aria-labelledby="user-posts" className="space-y-6">
      {posts.map(post => (
        <PostCard
          key={post.id}
          id={post.id}
          title={post.title}
          content={post.content}
          author={post.author}
          commentCount={post.commentCount}
        />
      ))}
      
      {hasMore && (
        <div className="flex justify-center">
          <Button 
            variant="outline" 
            onClick={loadMorePosts}
            disabled={paginationLoading}
          >
            {paginationLoading ? 'Loading...' : 'Load More Posts'}
          </Button>
        </div>
      )}
    </section>
  );
}

export default async function UserPage({ params }: UserPageProps) {
  try {
    // We just check if the user exists
    await getUserById(params.id);
  } catch (error) {
    notFound();
  }
  return (
    <div className="container mx-auto px-4 py-8">
      <Link href="/" className="text-sm text-muted-foreground hover:underline mb-6 inline-block">
        ← Back to feed
      </Link>
      
      <div className="grid gap-8 md:grid-cols-[1fr_2fr]">
        <aside>
          <Suspense fallback={<div aria-live="polite" aria-busy="true"><LoadingSkeleton count={1} /></div>}>
            <UserProfile userId={params.id} />
          </Suspense>
        </aside>
        
        <div>
          <h2 className="text-2xl font-semibold mb-6" id="user-posts">User Posts</h2>
          
          <Suspense fallback={<div aria-live="polite" aria-busy="true"><LoadingSkeleton count={3} /></div>}>
            <UserPosts userId={params.id} />
          </Suspense>
        </div>
      </div>
    </div>
  );
}


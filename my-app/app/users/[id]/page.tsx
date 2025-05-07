import { Suspense } from "react";
import Link from "next/link";
import { notFound } from "next/navigation";
import { UserInfo } from "@/components/ui/user-info";
import { LoadingSkeleton } from "@/components/shared/loading-skeleton";
import { getUserById, getFollowers, getFollowing, ApiError } from "@/lib/api";
import { UserPosts } from "./user-posts";

interface UserPageProps {
  params: {
    id: string;
  };
}

export async function generateMetadata({ params }: UserPageProps) {
  const { id } = await params;
  try {
    const user = await getUserById(id);
    
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

export default async function UserPage({ params }: UserPageProps) {
  const { id } = await params;
  
  try {
    // We just check if the user exists
    await getUserById(id);
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
            <UserProfile userId={id} />
          </Suspense>
        </aside>
        
        <div>
          <h2 className="text-2xl font-semibold mb-6" id="user-posts">User Posts</h2>
          
          <Suspense fallback={<div aria-live="polite" aria-busy="true"><LoadingSkeleton count={3} /></div>}>
            <UserPosts userId={id} />
          </Suspense>
        </div>
      </div>
    </div>
  );
}

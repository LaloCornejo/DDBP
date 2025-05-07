import { notFound } from "next/navigation";
import { Suspense } from "react";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { CommentSection } from "@/components/ui/comment-section";
import { LoadingSkeleton } from "@/components/shared/loading-skeleton";
import { getPostById, enrichPostWithDetails } from "@/lib/data";
import Link from "next/link";

interface PostPageProps {
  params: {
    id: string;
  };
}

export async function generateMetadata({ params }: PostPageProps) {
  const { id } = await params;
  const post = await getPostById(id);
  
  if (!post) {
    return {
      title: "Post Not Found",
      description: "The requested post could not be found.",
    };
  }
  
  return {
    title: `${post.title} - Social Feed`,
    description: post.content.substring(0, 160),
    openGraph: {
      title: `${post.title} - Social Feed`,
      description: post.content.substring(0, 160),
      type: "article",
      authors: [post.user_id],
      siteName: "Social Feed"
    }
  };
}

export default async function PostPage({ params }: PostPageProps) {
  const { id } = await params;
  const post = await getPostById(id);
  
  if (!post) {
    notFound();
  }
  
  const enrichedPost = await enrichPostWithDetails(post);
  
  // Add console.log for debugging
  console.log('Enriched Post:', JSON.stringify({
    id: enrichedPost.id,
    title: enrichedPost.title,
    author: enrichedPost.author,
    comments: enrichedPost.comments.map(c => ({
      id: c.id,
      author: c.author,
      content: c.content.substring(0, 50) + '...'
    }))
  }, null, 2));
  
  // Add error boundary for author fallback
  const authorName = enrichedPost.author?.name || 'Unknown Author';
  const authorImage = enrichedPost.author?.image || '';
  
  return (
    <div className="container mx-auto px-4 py-8">
      <Link href="/" className="text-sm text-muted-foreground hover:underline mb-6 inline-block">
        ← Back to feed
      </Link>
      
      <Card className="mb-8">
        <CardHeader className="flex flex-row items-center gap-4">
          <Avatar>
            <AvatarImage src={authorImage} alt={authorName} />
            <AvatarFallback>
              {authorName.substring(0, 2).toUpperCase()}
            </AvatarFallback>
          </Avatar>
          <div className="flex flex-col">
            <h1 className="text-2xl font-bold" id="post-title">{enrichedPost.title}</h1>
            <Link 
              href={`/users/${enrichedPost.author?.id}`} 
              className="text-sm text-muted-foreground hover:underline"
              aria-label={`View ${authorName}'s profile`}
            >
              by {authorName}
            </Link>
            <time className="text-xs text-muted-foreground" dateTime={enrichedPost.timestamp}>
              {new Date(enrichedPost.timestamp).toLocaleString()}
            </time>
          </div>
        </CardHeader>
        <CardContent>
          <article aria-labelledby="post-title">
            <p className="whitespace-pre-line">{enrichedPost.content}</p>
          </article>
        </CardContent>
      </Card>
      
      <Suspense fallback={<div aria-live="polite" aria-busy="true"><LoadingSkeleton count={2} /></div>}>
        <section aria-label="Comments">
          <CommentSection comments={enrichedPost.comments} />
        </section>
      </Suspense>
    </div>
  );
}


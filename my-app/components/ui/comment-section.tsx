"use client";

import { cn } from "@/lib/utils";
import { Card, CardContent, CardHeader } from "./card";
import { Avatar, AvatarFallback, AvatarImage } from "./avatar";
import { Separator } from "./separator";
import Link from "next/link";

type Comment = {
  id: string;
  content: string;
  author: {
    id: string;
    name: string;
    image?: string;
  };
  timestamp: string;
};

interface CommentSectionProps {
  comments: Comment[];
  className?: string;
}

export function CommentSection({ comments, className }: CommentSectionProps) {
  return (
    <div className={cn("space-y-6", className)}>
      <h2 className="text-2xl font-semibold">Comments ({comments.length})</h2>
      {comments.length === 0 ? (
        <p className="text-muted-foreground">No comments yet.</p>
      ) : (
        <div className="space-y-4">
          {comments.map((comment) => (
            <Card key={comment.id} className="border-muted">
              <CardHeader className="flex flex-row items-center gap-4 p-4">
                <Avatar className="h-8 w-8">
                  <AvatarImage src={comment.author.image} alt={comment.author.name} />
                  <AvatarFallback>
                    {comment.author.name.substring(0, 2).toUpperCase()}
                  </AvatarFallback>
                </Avatar>
                <div className="flex flex-1 flex-col">
                  <Link href={`/users/${comment.author.id}`} className="font-medium hover:underline">
                    {comment.author.name}
                  </Link>
                  <time className="text-xs text-muted-foreground" dateTime={comment.timestamp}>
                    {new Date(comment.timestamp).toLocaleString()}
                  </time>
                </div>
              </CardHeader>
              <Separator />
              <CardContent className="p-4 pt-3">
                <p>{comment.content}</p>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}


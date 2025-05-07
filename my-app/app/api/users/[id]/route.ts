import { NextResponse } from "next/server";
import { getUserById, getPostsByAuthorId } from "@/lib/data";

export async function GET(
  request: Request,
  { params }: { params: { id: string } }
) {
  try {
    const { id } = params;
    
    // Simulate a slight delay to demonstrate loading states
    await new Promise(resolve => setTimeout(resolve, 500));
    
    const user = getUserById(id);
    
    if (!user) {
      return NextResponse.json(
        { error: "User not found" },
        { status: 404 }
      );
    }
    
    // Get posts by this user
    const userPosts = getPostsByAuthorId(id).map(post => ({
      id: post.id,
      title: post.title,
      content: post.content.substring(0, 150) + (post.content.length > 150 ? '...' : ''),
      timestamp: post.timestamp,
      commentCount: post.commentCount
    }));
    
    return NextResponse.json({
      user,
      posts: userPosts
    });
  } catch (error) {
    console.error("Error fetching user:", error);
    return NextResponse.json(
      { error: "Failed to fetch user" },
      { status: 500 }
    );
  }
}


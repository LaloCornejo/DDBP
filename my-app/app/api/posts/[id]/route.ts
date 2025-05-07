import { NextResponse } from "next/server";
import { getPostById, enrichPostWithDetails } from "@/lib/data";

export async function GET(
  request: Request,
  { params }: { params: { id: string } }
) {
  try {
    const { id } = params;
    
    // Simulate a slight delay to demonstrate loading states
    await new Promise(resolve => setTimeout(resolve, 500));
    
    const post = await getPostById(id);
    
    if (!post) {
      return NextResponse.json(
        { error: "Post not found" },
        { status: 404 }
      );
    }
    
    const enrichedPost = await enrichPostWithDetails(post);
    
    return NextResponse.json(enrichedPost);
  } catch (error) {
    console.error("Error fetching post:", error);
    return NextResponse.json(
      { error: "Failed to fetch post" },
      { status: 500 }
    );
  }
}


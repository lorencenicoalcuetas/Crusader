import { NextRequest, NextResponse } from "next/server";

// PayMongo API for GCash payments
// Documentation: https://developers.paymongo.com/docs

const PAYMONGO_SECRET_KEY = process.env.PAYMONGO_SECRET_KEY || "";

interface CreatePaymentRequest {
  amount: number; // Amount in PHP centavos
  nftId: string;
  nftName: string;
  buyerAddress: string;
}

export async function POST(request: NextRequest) {
  try {
    const body: CreatePaymentRequest = await request.json();
    const { amount, nftId, nftName, buyerAddress } = body;

    if (!amount || !nftId || !nftName || !buyerAddress) {
      return NextResponse.json(
        { error: "Missing required fields" },
        { status: 400 }
      );
    }

    // If no PayMongo key, return mock response for demo
    if (!PAYMONGO_SECRET_KEY) {
      return NextResponse.json({
        success: true,
        checkoutUrl: `/payment/success?nftId=${nftId}&mock=true`,
        sourceId: `mock_${Date.now()}`,
        message: "Demo mode - PayMongo API key not configured",
      });
    }

    // Create a GCash source via PayMongo
    const response = await fetch("https://api.paymongo.com/v1/sources", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Basic ${Buffer.from(PAYMONGO_SECRET_KEY + ":").toString("base64")}`,
      },
      body: JSON.stringify({
        data: {
          attributes: {
            amount: amount, // Amount in centavos
            redirect: {
              success: `${process.env.NEXT_PUBLIC_BASE_URL || "http://localhost:3000"}/payment/success?nftId=${nftId}`,
              failed: `${process.env.NEXT_PUBLIC_BASE_URL || "http://localhost:3000"}/payment/failed?nftId=${nftId}`,
            },
            type: "gcash",
            currency: "PHP",
            metadata: {
              nftId,
              nftName,
              buyerAddress,
            },
          },
        },
      }),
    });

    const data = await response.json();

    if (!response.ok) {
      console.error("PayMongo error:", data);
      return NextResponse.json(
        { error: data.errors?.[0]?.detail || "Payment creation failed" },
        { status: response.status }
      );
    }

    return NextResponse.json({
      success: true,
      checkoutUrl: data.data.attributes.redirect.checkout_url,
      sourceId: data.data.id,
    });
  } catch (error) {
    console.error("Payment creation error:", error);
    return NextResponse.json(
      { error: "Internal server error" },
      { status: 500 }
    );
  }
}

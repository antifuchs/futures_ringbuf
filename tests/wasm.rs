#![ cfg( target_arch = "wasm32" )]
#![ feature( async_await )]


// Verify basic functionality on wasm
//
// Tested:
//
// - ✔ the code from the basic example


use
{
	wasm_bindgen_test :: { *                                 } ,
	wasm_bindgen      :: { UnwrapThrowExt                    } ,
	async_runtime     :: { *                                 } ,
	futures_ringbuf   :: { *                                 } ,
	futures           :: { SinkExt, StreamExt, future::ready } ,
	futures_codec     :: { Framed, LinesCodec                } ,
};

wasm_bindgen_test_configure!(run_in_browser);



#[wasm_bindgen_test]
//
fn basic_example()
{
	let mock = RingBuffer::new( 13 );
	let (mut writer, reader) = Framed::new( mock, LinesCodec{} ).split();


	let send_task = async move
	{
		writer.send( "Hello World\n".to_string() ).await.expect( "send" );
		writer.send( "Second line\n".to_string() ).await.expect( "send" );
		writer.close().await.expect( "close sender" );
	};


	let receive_task = async move
	{
		let count = reader.fold( 0, |count, _| ready( count + 1 ) ).await;

		assert_eq!( count, 2 );
	};

	rt::spawn( send_task    ).expect_throw( "spawn future" );
	rt::spawn( receive_task ).expect_throw( "spawn future" );
}



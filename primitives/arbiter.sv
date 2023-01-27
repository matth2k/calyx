
module arbiter_2_sdp #(
    parameter WIDTH = 32,
    parameter SIZE = 16,
    parameter IDX_SIZE = 4
) (
   // Common signals
   input wire logic clk,
   input wire logic reset,
   input wire logic [IDX_SIZE-1:0] addr0,
   input wire logic [IDX_SIZE-1:0] addr1,
   input wire logic write_en0,
   input wire logic [WIDTH-1:0] in0,
   input wire logic write_en1,
   input wire logic [WIDTH-1:0] in1,
   // Read ports
   input wire logic read_en0,
   input wire logic read_en1,
   // Dynamic port memory
   input wire logic mem_write_done,
   input wire logic mem_read_done,
   input wire logic [WIDTH-1:0] mem_out,

   output logic [WIDTH-1:0] out0,
   output logic [WIDTH-1:0] out1,
   output logic write_done0,
   output logic read_done0,
   output logic write_done1,
   output logic read_done1,
   // Dynamic port memory
   output wire logic [IDX_SIZE-1:0] mem_addr,
   // Write ports
   output wire logic mem_write_en,
   output wire logic [WIDTH-1:0] mem_in,
   // Read ports
   output wire logic mem_read_en
);
  
  logic preference;

  wire logic [1:0] fsm_next = (preference && port0_req)? 2'd1 : port1_req? 2'd2 : port0_req? 2'd1 : 2'd0;
  logic [1:0] fsm_current;
  wire logic port0_req = read_en0 | write_en0;
  wire logic port1_req = read_en1 | write_en1;
  wire logic port0_busy = fsm_current == 2'd1;
  wire logic port1_busy = fsm_current == 2'd2;
  wire logic arbiter_idle = fsm_current == 2'd0;
  wire logic arbiter_busy = ~arbiter_idle;

  assign mem_read_en = (port0_busy && read_en0) || (port1_busy && read_en1);
  assign mem_write_en = (port0_busy && write_en0) || (port1_busy && write_en1);
  assign mem_addr = port0_busy? addr0 : port1_busy? addr1 : 'd0;
  assign mem_in = (port0_busy && write_en0)? in0 : (port1_busy && write_en1)? in1 : 'd0;
  
  always_ff @(posedge clk) begin
    if (reset) begin
      out0 <= 'b0;
      out1 <= 'b0;
      write_done0 <= 1'b0;
      read_done0 <= 1'b0;
      write_done1 <= 1'b0;
      read_done1 <= 1'b0;
    end else begin
      
      if (port0_busy)
        preference <= 1'b1;
      else if (port1_busy)
        preference <= 1'b0;
      
      
      if (arbiter_idle) begin
        fsm_current <= fsm_next;
        write_done0 <= 1'b0;
        read_done0 <= 1'b0;
        write_done1 <= 1'b0;
        read_done1 <= 1'b0;
      end else if (port0_busy) begin
        if (mem_read_done) begin
          read_done0 <= 1'b1;
          out0 <= mem_out;
          fsm_current <= 'd0;
        end else if (mem_write_done) begin
          write_done0 <= 1'b1;
          fsm_current <= 'd0;
        end
      end else if (port1_busy) begin
        if (mem_read_done) begin
          read_done1 <= 1'b1;
          out1 <= mem_out;
          fsm_current <= 'd0;
        end else if (mem_write_done) begin
          write_done1 <= 1'b1;
          fsm_current <= 'd0;
        end
      end
    end
  end

  int fd;
  initial begin
    fd = $fopen("/home/matth2k/calyx/arb_out.log", "w");
  end
  always_ff @(posedge clk) begin
    $fdisplay(fd, "port0_go: %b, port1_go: %b", write_en0 | read_en0, write_en1 | read_en1);
    $fdisplay(fd, "port0_done: %b, port1_done: %b", write_done0 | read_done0, write_done1 | read_done1);
    $fdisplay(fd, "mem_go: %b, mem_done: %b", mem_read_en | mem_write_en, mem_read_done | mem_write_done);
  end

endmodule

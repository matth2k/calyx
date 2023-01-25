
module arbiter_2 #(
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
   output logic [IDX_SIZE-1:0] mem_addr,
   // Write ports
   output logic mem_write_en,
   output logic [WIDTH-1:0] mem_in,
   // Read ports
   output logic mem_read_en
);
  
  logic [1:0] read_fsm;
  // 0 - idle port 0 prefer
  // 1 - idle port 1 prefer
  // 2 - busy port 0
  // 3 - busy port 1

  wire logic arbiter_read_busy = read_fsm > 2'd1;
  wire logic arbiter_read_idle = !arbiter_read_busy;
  wire logic read_fsm_state0 = read_fsm == 2'd0;
  wire logic read_fsm_state1 = read_fsm == 2'd1;
  wire logic read_fsm_state2 = read_fsm == 2'd2;
  wire logic read_fsm_state3 = read_fsm == 2'd3;
  
  always_ff @(posedge clk) begin
    if (reset) begin
      read_fsm <= 2'd0;
    end else if (read_fsm == 2'd0) begin
      if (read_en0) begin
        read_fsm <= 2'd2;
      end else if (read_en1) begin
        read_fsm <= 2'd3;
      end
    end else if (read_fsm == 2'd1) begin
      if (read_en1) begin
        read_fsm <= 2'd3;
      end else if (read_en0) begin
        read_fsm <= 2'd2;
      end
    end else if (read_fsm == 2'd2) begin
      if (mem_read_done) begin
        read_fsm <= 2'd1;
      end
    end else if (read_fsm == 2'd3) begin
      if (mem_read_done) begin
        read_fsm <= 2'd0;
      end
    end
  end


  logic [1:0] write_fsm;
  // 0 - idle port 0 prefer
  // 1 - idle port 1 prefer
  // 2 - busy port 0
  // 3 - busy port 1

  wire logic arbiter_write_busy = write_fsm > 2'd1;
  wire logic arbiter_write_idle = !arbiter_write_busy;
  wire logic write_fsm_state0 = write_fsm == 2'd0;
  wire logic write_fsm_state1 = write_fsm == 2'd1;
  wire logic write_fsm_state2 = write_fsm == 2'd2;
  wire logic write_fsm_state3 = write_fsm == 2'd3;

  logic lastOpRead;
  always_ff @(posedge clk) begin
    if (arbiter_read_busy)
      lastOpRead <= 1'b1;
    else if (arbiter_write_busy)
      lastOpRead <= 1'b0;
  end

  wire logic stallWrite = arbiter_read_busy;
  
  always_ff @(posedge clk) begin
    if (reset) begin
      write_fsm <= 2'd0;
    end else if (write_fsm == 2'd0 && !stallWrite) begin
      if (write_en0) begin
        write_fsm <= 2'd2;
      end else if (write_en1) begin
        write_fsm <= 2'd3;
      end
    end else if (write_fsm == 2'd1 && !stallWrite) begin
      if (write_en1) begin
        write_fsm <= 2'd3;
      end else if (write_en0) begin
        write_fsm <= 2'd2;
      end
    end else if (write_fsm == 2'd2) begin
      if (mem_write_done) begin
        write_fsm <= 2'd1;
      end
    end else if (write_fsm == 2'd3) begin
      if (mem_write_done) begin
        write_fsm <= 2'd0;
      end
    end
  end


  always_ff @(posedge clk) begin
    if (reset) begin
      read_done0 <= 1'b0;
      read_done1 <= 1'b0;
      out0 <= 'd0;
      out1 <= 'd0;
      mem_read_en <= 'd0;
    end else begin
      if (arbiter_read_busy) begin
        if (mem_read_done) begin
          if (read_fsm_state2) begin
            read_done0 <= 1'b1;
            out0 <= mem_out;
          end else if (read_fsm_state3) begin
            read_done1 <= 1'b1;
            out1 <= mem_out;
          end
          mem_read_en <= 1'b0;
        end
      end else begin
        if (read_fsm_state0) begin
          if (read_en0) begin
            mem_read_en <= 1'b1;
            read_done0 <= 1'b0;
            mem_addr <= addr0;
          end else if (read_en1) begin
            mem_read_en <= 1'b1;
            read_done1 <= 1'b0;
            mem_addr <= addr1;
          end
        end else if (read_fsm_state1) begin
          if (read_en1) begin
            mem_read_en <= 1'b1;
            read_done1 <= 1'b0;
            mem_addr <= addr1;
          end else if (read_en0) begin
            mem_read_en <= 1'b1;
            read_done0 <= 1'b0;
            mem_addr <= addr0;
          end
        end
      end
    end
  end


  always_ff @(posedge clk) begin
    if (reset) begin
      write_done0 <= 1'b0;
      write_done1 <= 1'b0;
      mem_in <= 'd0;
      mem_write_en <= 'd0;
    end else begin
      if (arbiter_write_busy) begin
        if (mem_write_done) begin
          if (write_fsm_state2) begin
            write_done0 <= 1'b1;
          end else if (write_fsm_state3) begin
            write_done1 <= 1'b1;
          end
          mem_write_en <= 1'b0;
        end
      end else if (!stallWrite) begin
        if (write_fsm_state0) begin
          if (write_en0) begin
            mem_write_en <= 1'b1;
            write_done0 <= 1'b0;
            mem_addr <= addr0;
            mem_in <= in0;
          end else if (write_en1) begin
            mem_write_en <= 1'b1;
            write_done1 <= 1'b0;
            mem_addr <= addr1;
            mem_in <= in1;
          end
        end else if (write_fsm_state1) begin
          if (write_en1) begin
            mem_write_en <= 1'b1;
            write_done1 <= 1'b0;
            mem_addr <= addr1;
            mem_in <= in1;
          end else if (write_en0) begin
            mem_write_en <= 1'b1;
            write_done0 <= 1'b0;
            mem_addr <= addr0;
            mem_in <= in0;
          end
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
    $fdisplay(fd, "read_fsm: %d, write_fsm: %d", read_fsm, write_fsm);
  end

endmodule

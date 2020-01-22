# frozen_string_literal: true

module Inkoc
  module TypeSystem
    class Error
      include Type
      include TypeWithAttributes

      def attributes(name)
        SymbolTable.new
      end

      def lookup_attribute(name)
        NullSymbol.new(name)
      end

      def define_attribute(name, *)
        NullSymbol.new(name)
      end

      def error?
        true
      end

      def type_compatible?(*)
        true
      end

      def type_name
        '<type error>'
      end

      def type_instance_of?(other)
        other.is_a?(self.class)
      end

      def lookup_type(*)
        self
      end

      def type_instance?
        false
      end

      def new_instance(*)
        self
      end

      def base_type
        self
      end
    end
  end
end
